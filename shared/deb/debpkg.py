#!/usr/bin/env python3
"""Build one Debian package per plugin of a marketplace, plus <namespace>-marketplace.

Runs inside the pinned build image that build-debs.sh starts, on a marketplace
distribution such as ./create-dist.sh writes: `.claude-plugin/marketplace.json`
and the plugin directories its entries point at. A plugin's `deb/` directory is
its packaging metadata; the package does not install it.

The namespace (`codexis` for this repo) prefixes the package names and names the
directory the marketplace installs into, /usr/share/cdx-plugins/<namespace>/.
Installed, that directory is the distribution: every package puts its files
where the distribution has them. A plugin package installs:
  <namespace dir>/<source>/  the plugin tree, at its marketplace.json source
  /usr/bin/<command>         each bin/ entry, invokable on PATH

An ELF command is a real file in /usr/bin and the plugin's bin/ entry links to
it. A script command stays in the plugin tree and /usr/bin links to it: the
Python entry points find their lib/ next to their own resolved path. Each tree
carries cdx-manifest.json, which lists the files the daemon loads from it.

Depends is derived from the package contents (dpkg-shlibdeps for ELF files,
shebangs for scripts) and plugin.json `dependencies`; `deb/control` adds what
neither can see, such as a tool a binary spawns. `deb/wheels` names pure-Python
libraries Ubuntu does not package; they are unpacked into the plugin's lib/.

<namespace>-marketplace installs the rest of the distribution (marketplace.json
and the files beside it) and REVISION_FILE, and depends on every plugin package,
so one `apt install codexis-marketplace` installs the whole marketplace.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import textwrap
import time
import urllib.request
import zipfile
from collections.abc import Iterator
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath

PLUGINS_ROOT = PurePosixPath("usr/share/cdx-plugins")
BIN_DIR = PurePosixPath("usr/bin")
DEBIAN_REVISION = "1"
MANIFEST = "cdx-manifest.json"
# The commit the marketplace was built from, which the daemon reads to tell a
# release from the one before. The marketplace volume carries it too.
REVISION_FILE = ".codexis-marketplace-revision"
MANIFEST_SCHEMA = 1

# Program a shebang runs -> the package providing it. None for programs from
# Essential packages, which a package must not depend on.
INTERPRETERS: dict[str, str | None] = {
    "bash": None,
    "sh": None,
    "python3": "python3",
    "node": "nodejs",
}

ELF_ARCHITECTURES = {0x3E: "amd64", 0xB7: "arm64"}

_REQUIRE = re.compile(rb"""\brequire\(\s*["']([^"']+)["']\s*\)""")


class PackagingError(Exception):
    pass


@dataclass
class Package:
    name: str
    version: str
    architecture: str
    maintainer: str
    synopsis: str
    description: str
    depends: list[str]
    plugin: str | None = None
    # Where the plugin tree installs, absolute; None for the marketplace package.
    tree: PurePosixPath | None = None
    section: str = "misc"
    commands: list[str] = field(default_factory=list)
    elf_files: list[Path] = field(default_factory=list)
    python_modules: bool = False
    shlibs: list[str] = field(default_factory=list)

    @property
    def filename(self) -> str:
        return f"{self.name}_{self.version}_{self.architecture}.deb"


def install_root(namespace: str) -> PurePosixPath:
    """The directory the plugin trees of `namespace` install into, relative to /."""
    return PLUGINS_ROOT / namespace


def package_name(namespace: str, plugin: str) -> str:
    """<namespace>-<plugin>, without doubling the prefix of a plugin named after it."""
    if plugin == namespace or plugin.startswith(f"{namespace}-"):
        return plugin
    return f"{namespace}-{plugin}"


def elf_machine(path: Path) -> int | None:
    """The ELF e_machine of `path`, None when it is not an ELF file."""
    with path.open("rb") as f:
        head = f.read(20)
    if len(head) < 20 or head[:4] != b"\x7fELF":
        return None
    return int.from_bytes(head[18:20], "little" if head[5] == 1 else "big")


def interpreter(path: Path) -> str | None:
    """The program a script's shebang runs, None when `path` is no script."""
    with path.open("rb") as f:
        line = f.readline(256)
    if not line.startswith(b"#!"):
        return None
    words = line[2:].decode("utf-8", "replace").split()
    if words and PurePosixPath(words[0]).name == "env":
        words = [w for w in words[1:] if not w.startswith("-")]
    if not words:
        raise PackagingError(f"{path}: shebang names no program")
    return PurePosixPath(words[0]).name


def unbundled_modules(path: Path) -> list[str]:
    """Modules a Node script requires that are neither built in nor relative."""
    found = {m.decode() for m in _REQUIRE.findall(path.read_bytes())}
    return sorted(m for m in found if not m.startswith(("node:", "./", "../")))


def split_relations(value: str) -> list[str]:
    return [" ".join(r.split()) for r in value.split(",") if r.strip()]


def _lines(path: Path) -> Iterator[tuple[int, str]]:
    """(number, line) of the non-blank, non-comment lines of `path`."""
    if not path.is_file():
        return
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if line.strip() and not line.startswith("#"):
            yield number, line


def read_depends(plugin_dir: Path) -> list[str]:
    """The Depends field of `<plugin>/deb/control` (deb822, may be folded)."""
    path = plugin_dir / "deb" / "control"
    value = None
    for number, line in _lines(path):
        if line[0] in " \t" and value is not None:
            value += " " + line.strip()
            continue
        name, sep, rest = line.partition(":")
        if not sep or name != "Depends" or value is not None:
            raise PackagingError(f"{path}:{number}: expected a single Depends field")
        value = rest
    return split_relations(value or "")


def read_links(plugin_dir: Path) -> list[tuple[PurePosixPath, PurePosixPath]]:
    """Extra symlinks from `<plugin>/deb/links`, in dh_link format."""
    path = plugin_dir / "deb" / "links"
    links = []
    for number, line in _lines(path):
        parts = line.split()
        if len(parts) != 2:
            raise PackagingError(f"{path}:{number}: expected '<target> <link>'")
        target, link = (PurePosixPath(p.lstrip("/")) for p in parts)
        links.append((target, link))
    return links


def read_wheels(plugin_dir: Path) -> list[tuple[str, str]]:
    """(url, sha256) of each wheel `<plugin>/deb/wheels` names."""
    path = plugin_dir / "deb" / "wheels"
    wheels = []
    for number, line in _lines(path):
        parts = line.split()
        if len(parts) != 2 or not parts[1].startswith("sha256="):
            raise PackagingError(f"{path}:{number}: expected '<url> sha256=<hex>'")
        url, digest = parts[0], parts[1].removeprefix("sha256=")
        if not url.endswith("-none-any.whl"):
            raise PackagingError(f"{path}:{number}: only pure-Python wheels")
        wheels.append((url, digest))
    return wheels


def vendor_wheels(plugin_dir: Path, tree: Path) -> None:
    """Unpack the wheels `deb/wheels` names into the plugin's lib/.

    lib/ is on sys.path for every Python entry point of a plugin, so a library
    placed there imports as if installed. The .dist-info stays for its license.
    """
    for url, digest in read_wheels(plugin_dir):
        with urllib.request.urlopen(url) as response:
            data = response.read()
        if hashlib.sha256(data).hexdigest() != digest:
            raise PackagingError(f"{url}: sha256 mismatch")
        with tempfile.TemporaryFile() as archive:
            archive.write(data)
            with zipfile.ZipFile(archive) as wheel:
                members = [
                    m
                    for m in wheel.namelist()
                    if not PurePosixPath(m).parts[0].endswith(".data")
                ]
                wheel.extractall(tree / "lib", members)


def _symlink(link: Path, target: Path) -> None:
    link.parent.mkdir(parents=True, exist_ok=True)
    link.symlink_to(os.path.relpath(target, link.parent))


def _files(root: Path) -> list[Path]:
    """Regular files under `root`, symlinks excluded, in a stable order."""
    return sorted(p for p in root.rglob("*") if p.is_file() and not p.is_symlink())


def _unique(items: list[str]) -> list[str]:
    return list(dict.fromkeys(items))


def _normalize_modes(root: Path) -> None:
    for path in [root, *root.rglob("*")]:
        if path.is_symlink():
            continue
        if path.is_dir():
            path.chmod(0o755)
        else:
            path.chmod(0o755 if path.stat().st_mode & 0o111 else 0o644)


def _install_commands(root: Path, tree: Path, plugin_dir: Path) -> list[str]:
    usr_bin = root / BIN_DIR
    bin_dir = tree / "bin"
    entries = sorted(bin_dir.iterdir()) if bin_dir.is_dir() else []
    for entry in entries:
        if entry.is_symlink() or not entry.is_file():
            continue
        command = usr_bin / entry.name
        if elf_machine(entry) is None:
            _symlink(command, entry)
        else:
            usr_bin.mkdir(parents=True, exist_ok=True)
            shutil.move(entry, command)
            _symlink(entry, command)
    for target, link in read_links(plugin_dir):
        if not (root / target).is_file():
            raise PackagingError(
                f"{plugin_dir.name}: link target /{target} not packaged"
            )
        _symlink(root / link, root / target)
        # A script the plugin ships as data becomes a command here, as its
        # install hook made it one with `install -m 0755`.
        if link.parent == BIN_DIR:
            (root / target).chmod(0o755)
    return sorted(p.name for p in usr_bin.iterdir()) if usr_bin.is_dir() else []


def _architecture(files: list[Path]) -> tuple[str, list[Path]]:
    machines: dict[int, Path] = {}
    elf_files = []
    for path in files:
        machine = elf_machine(path)
        if machine is None:
            continue
        if machine not in ELF_ARCHITECTURES:
            raise PackagingError(f"{path}: unsupported ELF machine {machine:#x}")
        machines.setdefault(machine, path)
        elf_files.append(path)
    if len(machines) > 1:
        found = ", ".join(f"{p} ({ELF_ARCHITECTURES[m]})" for m, p in machines.items())
        raise PackagingError(f"ELF files for more than one architecture: {found}")
    arch = ELF_ARCHITECTURES[next(iter(machines))] if machines else "all"
    return arch, elf_files


def _interpreter_depends(files: list[Path]) -> list[str]:
    depends = []
    for path in files:
        if not path.stat().st_mode & 0o111:
            continue
        program = interpreter(path)
        if program is None:
            continue
        if program not in INTERPRETERS:
            raise PackagingError(
                f"{path}: no package known for interpreter '{program}' "
                "- add it to INTERPRETERS"
            )
        if program == "node" and (missing := unbundled_modules(path)):
            raise PackagingError(
                f"{path}: requires modules the bundle does not contain: "
                f"{', '.join(missing)}"
            )
        if package := INTERPRETERS[program]:
            depends.append(package)
    return sorted(set(depends))


def _within(path: str, context: str, container: str) -> PurePosixPath:
    """`path` normalized; refused when it leaves the directory it is relative to."""
    rel = PurePosixPath(os.path.normpath(path))
    if rel.is_absolute() or rel.parts[:1] == ("..",):
        raise PackagingError(f"{context}: {path} is outside the {container}")
    return rel


def _declared_dirs(tree: Path, plugin_json: dict, key: str) -> list[Path]:
    """The directories plugin.json `key` names (a path or a list), ./<key> if none."""
    value = plugin_json.get(key, f"./{key}")
    return [
        tree / _within(path, f"plugin.json {key}", "plugin")
        for path in ([value] if isinstance(value, str) else value)
    ]


def plugin_manifest(tree: Path, plugin_json: dict, version: str, category: str) -> dict:
    """cdx-manifest.json of the plugin at `tree`: what the daemon loads from it.

    Each entry is the path of a file the daemon parses, relative to the plugin
    directory, so that file stays the only source of what it says. A kind the
    plugin does not have is left out, and so are hooks: dpkg does what the
    lifecycle hooks did.
    """

    def within(key: str, pattern: str) -> list[Path]:
        return [
            p for d in _declared_dirs(tree, plugin_json, key) for p in d.glob(pattern)
        ]

    manifest: dict = {
        "schemaVersion": MANIFEST_SCHEMA,
        "name": plugin_json["name"],
        "version": version,
        "category": category,
        "plugin": ".claude-plugin/plugin.json",
    }
    if (tree / "README.md").is_file():
        manifest["readme"] = "README.md"
    kinds = {
        "skills": within("skills", "*/SKILL.md"),
        "agents": within("agents", "*.md"),
        "apps": within("components", "*/component.json"),
        "previews": list(tree.glob("previews/preview.json")),
        "automations": [
            *tree.glob("automations/*.md"),
            *tree.glob("automation-templates/*.md"),
        ],
    }
    for kind, paths in kinds.items():
        if listed := sorted(
            {p.relative_to(tree).as_posix() for p in paths if p.is_file()}
        ):
            manifest[kind] = listed
    if (tree / ".mcp.json").is_file():
        manifest["mcpServers"] = ".mcp.json"
    return manifest


def source_path(marketplace_dir: Path, entry: dict) -> PurePosixPath:
    """Where a marketplace.json entry's plugin is, relative to the marketplace."""
    name, source = entry["name"], entry.get("source")
    if not isinstance(source, str):
        raise PackagingError(
            f"marketplace.json {name}: source {json.dumps(source)} "
            "is not a directory of the marketplace"
        )
    location = _within(source, f"marketplace.json {name}", "marketplace")
    if not (marketplace_dir / location / ".claude-plugin" / "plugin.json").is_file():
        raise PackagingError(
            f"marketplace.json {name}: {source} holds no .claude-plugin/plugin.json"
        )
    return location


def lay_out(
    marketplace_dir: Path,
    location: PurePosixPath,
    root: Path,
    namespace: str,
    category: str,
) -> Package:
    """Arrange the package filesystem of the plugin at `location` under `root`."""
    plugin_dir = marketplace_dir / location
    plugin_json = json.loads(
        (plugin_dir / ".claude-plugin" / "plugin.json").read_text()
    )
    name = plugin_json["name"]
    version = f"{plugin_json['version']}-{DEBIAN_REVISION}"
    installed = install_root(namespace) / location
    tree = root / installed
    shutil.copytree(plugin_dir, tree, symlinks=True)
    # The packaging metadata is read from plugin_dir; it is not installed.
    shutil.rmtree(tree / "deb", ignore_errors=True)

    vendor_wheels(plugin_dir, tree)
    commands = _install_commands(root, tree, plugin_dir)
    manifest = plugin_manifest(tree, plugin_json, version, category)
    (tree / MANIFEST).write_text(json.dumps(manifest, indent=2) + "\n")
    _normalize_modes(root)
    files = _files(root)
    architecture, elf_files = _architecture(files)
    author = plugin_json["author"]
    return Package(
        plugin=name,
        tree=PurePosixPath("/") / installed,
        name=package_name(namespace, name),
        version=version,
        architecture=architecture,
        maintainer=f"{author['name']} <{author['email']}>",
        synopsis=plugin_json.get("i18n", {}).get("en", {}).get("displayName", name),
        description=plugin_json["description"],
        depends=_unique(
            _interpreter_depends(files)
            + [package_name(namespace, d) for d in plugin_json.get("dependencies", [])]
            + read_depends(plugin_dir)
        ),
        commands=commands,
        elf_files=elf_files,
        python_modules=any(tree.rglob("*.py")),
    )


def lay_out_marketplace(
    marketplace_dir: Path,
    locations: list[PurePosixPath],
    root: Path,
    namespace: str,
    revision: str,
) -> None:
    """Arrange the marketplace's own files under `root`: all but its plugins."""
    plugins = {marketplace_dir / location for location in locations}

    def ignore(directory: str, names: list[str]) -> list[str]:
        here = Path(directory)
        return [
            name
            for name in names
            if here / name in plugins or (here == marketplace_dir and name == ".git")
        ]

    target = root / install_root(namespace)
    shutil.copytree(marketplace_dir, target, symlinks=True, ignore=ignore)
    (target / REVISION_FILE).write_text(revision + "\n")
    _normalize_modes(root)


def marketplace_package(
    marketplace: dict, packages: list[Package], namespace: str, epoch: int
) -> Package:
    """<namespace>-marketplace, with a Depends on every plugin package.

    The marketplace has no version of its own, so the time of the commit it was
    built from is one: it grows with every release, which is all apt needs to
    upgrade it.
    """
    name = f"{namespace}-marketplace"
    owner = marketplace["owner"]
    english = marketplace.get("i18n", {}).get("en", {})
    return Package(
        name=name,
        version=time.strftime("%Y%m%d.%H%M%S", time.gmtime(epoch)),
        architecture="all",
        maintainer=f"{owner['name']} <{owner['email']}>",
        synopsis=english.get("displayName", name),
        description=english.get("description", ""),
        depends=sorted(f"{p.name} (>= {p.version})" for p in packages),
        section="metapackages",
    )


def shlib_depends(elf_files: list[Path]) -> list[str]:
    """Shared-library Depends of `elf_files`, as dpkg-shlibdeps computes them."""
    if not elf_files:
        return []
    with tempfile.TemporaryDirectory() as work:
        (Path(work) / "debian").mkdir()
        (Path(work) / "debian" / "control").write_text(
            "Source: cdx-plugins\n\nPackage: cdx-plugins\nArchitecture: any\n"
        )
        result = subprocess.run(
            ["dpkg-shlibdeps", "-O", *(f"-e{path}" for path in elf_files)],
            cwd=work,
            capture_output=True,
            text=True,
        )
    if result.returncode != 0:
        raise PackagingError(f"dpkg-shlibdeps failed:\n{result.stderr}")
    for line in result.stdout.splitlines():
        if line.startswith("shlibs:Depends="):
            return split_relations(line.partition("=")[2])
    return []


def installed_size(root: Path) -> int:
    """Kibibytes, counted the way dpkg-gencontrol counts them."""
    total = 0
    for path in root.rglob("*"):
        if path.is_symlink() or path.is_dir():
            total += 1
        else:
            total += -(-path.stat().st_size // 1024)
    return total


def control(package: Package, size: int) -> str:
    fields = [
        ("Package", package.name),
        ("Version", package.version),
        ("Architecture", package.architecture),
        ("Maintainer", package.maintainer),
        ("Installed-Size", str(size)),
        ("Depends", ", ".join(_unique(package.shlibs + package.depends))),
        ("Section", package.section),
        ("Priority", "optional"),
        ("Description", package.synopsis),
    ]
    lines = [f"{name}: {value}" for name, value in fields if value]
    lines += [f" {line}" for line in textwrap.wrap(package.description, 78)]
    return "\n".join(lines) + "\n"


def maintainer_scripts(package: Package) -> dict[str, str]:
    """The maintainer scripts of `package`, by name."""
    if not package.python_modules:
        return {}
    tree = package.tree
    prerm = textwrap.dedent(
        f"""\
        #!/bin/sh
        set -e
        # A command run as root leaves __pycache__ next to the Python modules;
        # dpkg does not own it, so it would keep the tree behind.
        [ ! -d {tree} ] || find {tree} -type d -name __pycache__ -prune -exec rm -rf {{}} +
        """
    )
    return {"prerm": prerm}


def md5sums(root: Path) -> str:
    return "".join(
        f"{hashlib.md5(path.read_bytes()).hexdigest()}  {path.relative_to(root)}\n"
        for path in _files(root)
    )


def write_metadata(package: Package, root: Path) -> None:
    # Both describe the payload only, so they are taken before DEBIAN/ exists.
    sums = md5sums(root)
    size = installed_size(root)
    debian = root / "DEBIAN"
    debian.mkdir(mode=0o755)
    (debian / "control").write_text(control(package, size))
    (debian / "control").chmod(0o644)
    if sums:
        (debian / "md5sums").write_text(sums)
        (debian / "md5sums").chmod(0o644)
    for name, text in maintainer_scripts(package).items():
        (debian / name).write_text(text)
        (debian / name).chmod(0o755)


def check_conflicts(packages: list[Package], namespace: str) -> None:
    """Fail when two packages share a name or would ship the same command."""
    names: set[str] = {f"{namespace}-marketplace"}
    owners: dict[str, str] = {}
    for package in packages:
        if package.name in names:
            raise PackagingError(f"package name {package.name} is taken twice")
        names.add(package.name)
        for command in package.commands:
            if command in owners:
                raise PackagingError(
                    f"/usr/bin/{command} is shipped by both "
                    f"{owners[command]} and {package.name}"
                )
            owners[command] = package.name


def build_deb(package: Package, root: Path, out_dir: Path) -> str:
    """Build `package` from `root` into `out_dir`; returns its control file."""
    package.shlibs = shlib_depends(package.elf_files)
    write_metadata(package, root)
    subprocess.run(
        [
            "dpkg-deb",
            "--root-owner-group",
            "-Zxz",
            "--build",
            str(root),
            str(out_dir / package.filename),
        ],
        check=True,
        stdout=subprocess.DEVNULL,
    )
    depends = ", ".join(_unique(package.shlibs + package.depends))
    print(f"{package.filename}  Depends: {depends or '-'}")
    return (root / "DEBIAN" / "control").read_text()


def write_index(out_dir: Path, controls: dict[str, str]) -> None:
    """Packages index of the .deb files in `out_dir`, by filename to control file.

    It makes `out_dir` a flat apt repository. Each entry is the control file as
    written: dpkg-scanpackages would reparse it and strip a 0xA0 byte ending a
    wrapped description line, splitting a UTF-8 character such as "à" in two.
    """
    entries = []
    for filename, text in sorted(controls.items()):
        data = (out_dir / filename).read_bytes()
        entries.append(
            text
            + f"Filename: ./{filename}\n"
            + f"Size: {len(data)}\n"
            + f"MD5sum: {hashlib.md5(data).hexdigest()}\n"
            + f"SHA1: {hashlib.sha1(data).hexdigest()}\n"
            + f"SHA256: {hashlib.sha256(data).hexdigest()}\n"
        )
    index = "\n".join(entries).encode()
    (out_dir / "Packages").write_bytes(index)
    # apt asks a flat repository for the compressed index first.
    (out_dir / "Packages.gz").write_bytes(gzip.compress(index, mtime=0))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Build one Debian package per plugin of a marketplace."
    )
    parser.add_argument(
        "--marketplace",
        type=Path,
        required=True,
        help="directory holding .claude-plugin/marketplace.json",
    )
    parser.add_argument(
        "--namespace",
        required=True,
        help="prefix of the package names and directory under /usr/share/cdx-plugins",
    )
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument(
        "--revision",
        required=True,
        help=f"commit the marketplace was built from, written to {REVISION_FILE}",
    )
    parser.add_argument(
        "plugins",
        nargs="*",
        help="plugin names (default: all of them and <namespace>-marketplace)",
    )
    args = parser.parse_args(argv)

    marketplace = json.loads(
        (args.marketplace / ".claude-plugin" / "marketplace.json").read_text()
    )
    entries = marketplace["plugins"]
    unknown = sorted(set(args.plugins) - {e["name"] for e in entries})
    if unknown:
        print(f"error: unknown plugins: {', '.join(unknown)}", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory() as work:
        try:
            # Every package is laid out before any is built, so a conflict
            # between two of them fails the run before a .deb is written.
            roots = []
            locations = []
            for entry in entries:
                plugin = entry["name"]
                if args.plugins and plugin not in args.plugins:
                    continue
                if not (category := entry.get("category")):
                    raise PackagingError(f"marketplace.json gives {plugin} no category")
                location = source_path(args.marketplace, entry)
                root = Path(work) / plugin
                root.mkdir(mode=0o755)
                package = lay_out(
                    args.marketplace, location, root, args.namespace, category
                )
                if package.plugin != plugin:
                    raise PackagingError(
                        f"marketplace.json lists {plugin}, "
                        f"its plugin.json names {package.plugin}"
                    )
                roots.append((package, root))
                locations.append(location)
            packages = [package for package, _ in roots]
            check_conflicts(packages, args.namespace)
            if not args.plugins:
                if "SOURCE_DATE_EPOCH" not in os.environ:
                    raise PackagingError("SOURCE_DATE_EPOCH must be set")
                epoch = int(os.environ["SOURCE_DATE_EPOCH"])
                meta_package = marketplace_package(
                    marketplace, packages, args.namespace, epoch
                )
                meta_root = Path(work) / meta_package.name
                meta_root.mkdir(mode=0o755)
                lay_out_marketplace(
                    args.marketplace,
                    locations,
                    meta_root,
                    args.namespace,
                    args.revision,
                )
                roots.append((meta_package, meta_root))
            controls = {
                package.filename: build_deb(package, root, args.out)
                for package, root in roots
            }
        except PackagingError as exc:
            print(f"error: {exc}", file=sys.stderr)
            return 1
    write_index(args.out, controls)
    return 0


if __name__ == "__main__":
    sys.exit(main())
