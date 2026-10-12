#!/usr/bin/env python3
"""Build one Debian package per marketplace plugin.

Runs inside the pinned build image that build-debs.sh starts. That script stages
what each plugin ships (git-tracked files minus dist-exclusions.txt) as the
payload, and mounts the repo for the per-plugin `deb/` metadata, which the dist
does not carry.

Every package installs:
  /usr/share/cdx-plugins/codexis/<plugin>/  the plugin tree the daemon loads
  /usr/bin/<command>                        each bin/ entry, invokable on PATH

An ELF command is a real file in /usr/bin and the plugin's bin/ entry links to
it. A script command stays in the plugin tree and /usr/bin links to it: the
Python entry points find their lib/ next to their own resolved path.

Depends is derived from the package contents (dpkg-shlibdeps for ELF files,
shebangs for scripts) and plugin.json `dependencies`; `deb/control` adds what
neither can see, such as a tool a binary spawns.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import textwrap
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath

INSTALL_ROOT = PurePosixPath("usr/share/cdx-plugins/codexis")
BIN_DIR = PurePosixPath("usr/bin")
PACKAGE_PREFIX = "codexis-"
DEBIAN_REVISION = "1"
FRAGMENT_FIELDS = ("Depends", "Recommends", "Suggests")

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
    plugin: str
    name: str
    version: str
    architecture: str
    maintainer: str
    synopsis: str
    description: str
    depends: list[str]
    recommends: list[str]
    suggests: list[str]
    commands: list[str]
    elf_files: list[Path]
    python_modules: bool
    shlibs: list[str] = field(default_factory=list)

    @property
    def filename(self) -> str:
        return f"{self.name}_{self.version}_{self.architecture}.deb"


def package_name(plugin: str) -> str:
    return PACKAGE_PREFIX + plugin


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


def read_fragment(meta_dir: Path) -> dict[str, list[str]]:
    """Relationship fields from `<plugin>/deb/control` (deb822, comments allowed)."""
    path = meta_dir / "deb" / "control"
    if not path.is_file():
        return {}
    raw: dict[str, str] = {}
    current = None
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip() or line.startswith("#"):
            continue
        if line[0] in " \t":
            if current is None:
                raise PackagingError(f"{path}:{number}: continuation without a field")
            raw[current] += " " + line.strip()
            continue
        name, sep, value = line.partition(":")
        if not sep or name not in FRAGMENT_FIELDS:
            raise PackagingError(
                f"{path}:{number}: expected one of {', '.join(FRAGMENT_FIELDS)}"
            )
        if name in raw:
            raise PackagingError(f"{path}:{number}: {name} given twice")
        raw[name] = value
        current = name
    return {name: split_relations(value) for name, value in raw.items()}


def read_links(meta_dir: Path) -> list[tuple[PurePosixPath, PurePosixPath]]:
    """Extra symlinks from `<plugin>/deb/links`, in dh_link format."""
    path = meta_dir / "deb" / "links"
    if not path.is_file():
        return []
    links = []
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.split()
        if len(parts) != 2:
            raise PackagingError(f"{path}:{number}: expected '<target> <link>'")
        target, link = (PurePosixPath(p.lstrip("/")) for p in parts)
        links.append((target, link))
    return links


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


def _install_commands(root: Path, tree: Path, meta_dir: Path) -> list[str]:
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
    for target, link in read_links(meta_dir):
        if not (root / target).is_file():
            raise PackagingError(f"{meta_dir.name}: link target /{target} not packaged")
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


def lay_out(plugin_dir: Path, meta_dir: Path, root: Path) -> Package:
    """Arrange the package filesystem of one plugin under `root`."""
    manifest = json.loads((plugin_dir / ".claude-plugin" / "plugin.json").read_text())
    name = manifest["name"]
    tree = root / INSTALL_ROOT / name
    shutil.copytree(plugin_dir, tree, symlinks=True)

    # dpkg installs and removes the files, so the marketplace lifecycle scripts
    # (which copy binaries into ~/.local/bin) have nothing left to do.
    for script in tree.glob("hooks/*.sh"):
        script.unlink()
    hooks = tree / "hooks"
    if hooks.is_dir() and not any(hooks.iterdir()):
        hooks.rmdir()

    commands = _install_commands(root, tree, meta_dir)
    _normalize_modes(root)
    files = _files(root)
    architecture, elf_files = _architecture(files)
    fragment = read_fragment(meta_dir)
    author = manifest["author"]
    return Package(
        plugin=name,
        name=package_name(name),
        version=f"{manifest['version']}-{DEBIAN_REVISION}",
        architecture=architecture,
        maintainer=f"{author['name']} <{author['email']}>",
        synopsis=manifest.get("i18n", {}).get("en", {}).get("displayName", name),
        description=manifest["description"],
        depends=_unique(
            _interpreter_depends(files)
            + [package_name(d) for d in manifest.get("dependencies", [])]
            + fragment.get("Depends", [])
        ),
        recommends=fragment.get("Recommends", []),
        suggests=fragment.get("Suggests", []),
        commands=commands,
        elf_files=elf_files,
        python_modules=any(tree.rglob("*.py")),
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
    depends = _unique(package.shlibs + package.depends)
    fields = [
        ("Package", package.name),
        ("Version", package.version),
        ("Architecture", package.architecture),
        ("Maintainer", package.maintainer),
        ("Installed-Size", str(size)),
        ("Depends", ", ".join(depends)),
        ("Recommends", ", ".join(package.recommends)),
        ("Suggests", ", ".join(package.suggests)),
        ("Section", "misc"),
        ("Priority", "optional"),
        ("Description", package.synopsis),
    ]
    lines = [f"{name}: {value}" for name, value in fields if value]
    lines += [f" {line}" for line in textwrap.wrap(package.description, 78)]
    return "\n".join(lines) + "\n"


def prerm(plugin: str) -> str:
    tree = PurePosixPath("/") / INSTALL_ROOT / plugin
    return (
        "#!/bin/sh\n"
        "set -e\n"
        "# A command run as root leaves __pycache__ next to the Python modules;\n"
        "# dpkg does not own it, so it would keep the plugin directories behind.\n"
        f"[ ! -d {tree} ] || "
        f"find {tree} -type d -name __pycache__ -prune -exec rm -rf {{}} +\n"
    )


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
    (debian / "md5sums").write_text(sums)
    if package.python_modules:
        script = debian / "prerm"
        script.write_text(prerm(package.plugin))
        script.chmod(0o755)
    for path in (debian / "control", debian / "md5sums"):
        path.chmod(0o644)


def check_commands(packages: list[Package]) -> None:
    """Fail when two packages would ship the same /usr/bin entry."""
    owners: dict[str, str] = {}
    for package in packages:
        for command in package.commands:
            if command in owners:
                raise PackagingError(
                    f"/usr/bin/{command} is shipped by both "
                    f"{owners[command]} and {package.name}"
                )
            owners[command] = package.name


def build_deb(package: Package, root: Path, out_dir: Path) -> None:
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


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Build one Debian package per marketplace plugin."
    )
    parser.add_argument("--payload", type=Path, required=True)
    parser.add_argument("--meta", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("plugins", nargs="*", help="plugin names (default: all)")
    args = parser.parse_args(argv)

    available = sorted(
        p.name
        for p in args.payload.iterdir()
        if (p / ".claude-plugin" / "plugin.json").is_file()
    )
    unknown = sorted(set(args.plugins) - set(available))
    if unknown:
        print(f"error: unknown plugins: {', '.join(unknown)}", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory() as work:
        try:
            # Every package is laid out before any is built, so a conflict
            # between two of them fails the run before a .deb is written.
            roots = {}
            for plugin in args.plugins or available:
                root = Path(work) / plugin
                root.mkdir(mode=0o755)
                package = lay_out(args.payload / plugin, args.meta / plugin, root)
                roots[package.name] = (package, root)
            check_commands([package for package, _ in roots.values()])
            for package, root in roots.values():
                build_deb(package, root, args.out)
                depends = ", ".join(_unique(package.shlibs + package.depends))
                print(f"{package.filename}  Depends: {depends or '-'}")
        except PackagingError as exc:
            print(f"error: {exc}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
