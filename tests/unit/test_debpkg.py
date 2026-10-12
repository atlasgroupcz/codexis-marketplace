"""Unit tests for the Debian packaging of plugins (shared/deb/debpkg.py).

Each test lays a small fixture plugin out the way the builder does. Nothing here
calls dpkg-deb or dpkg-shlibdeps; shared/deb/smoke-test.sh covers the built
packages end to end.
"""

from __future__ import annotations

import hashlib
import json
import os
import stat
import subprocess
import zipfile
from pathlib import Path, PurePosixPath

import pytest

from shared.deb import debpkg
from shared.deb.debpkg import PackagingError

TREE = "usr/share/cdx-plugins/codexis"


def elf(machine: int = 0x3E) -> bytes:
    """An ELF header carrying `machine`, enough for the builder to classify."""
    return (
        b"\x7fELF\x02\x01\x01" + bytes(9) + b"\x02\x00" + machine.to_bytes(2, "little")
    )


def make_plugin(
    base: Path,
    name: str = "demo",
    files: dict[str, bytes | str] | None = None,
    executable: tuple[str, ...] = (),
    **manifest: object,
) -> Path:
    plugin = base / "payload" / name
    (plugin / ".claude-plugin").mkdir(parents=True)
    (plugin / ".claude-plugin" / "plugin.json").write_text(
        json.dumps(
            {
                "name": name,
                "version": "1.2.3",
                "description": "Demo plugin used by the packaging tests.",
                "author": {"name": "ATLAS GROUP", "email": "dev@example.com"},
                "i18n": {"en": {"displayName": "Demo - Packaging"}},
                **manifest,
            }
        )
    )
    for rel, content in (files or {}).items():
        path = plugin / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(content, bytes):
            path.write_bytes(content)
        else:
            path.write_text(content)
        path.chmod(0o755 if rel in executable else 0o644)
    return plugin


def meta(base: Path, name: str = "demo", **files: str) -> Path:
    directory = base / "repo" / "plugins" / name
    (directory / "deb").mkdir(parents=True)
    for filename, content in files.items():
        (directory / "deb" / filename).write_text(content)
    return directory


def lay_out(base: Path, plugin: Path, meta_dir: Path | None = None) -> debpkg.Package:
    root = base / "root" / plugin.name
    root.mkdir(parents=True)
    meta_dir = meta_dir or base / "repo" / "plugins" / plugin.name
    return debpkg.lay_out(plugin, meta_dir, root)


def test_elf_command_is_installed_in_usr_bin_and_linked_from_the_plugin(tmp_path):
    plugin = make_plugin(
        tmp_path, files={"bin/demo-cli": elf()}, executable=("bin/demo-cli",)
    )

    package = lay_out(tmp_path, plugin)

    root = tmp_path / "root" / "demo"
    command = root / "usr/bin/demo-cli"
    entry = root / TREE / "demo/bin/demo-cli"
    assert command.is_file() and not command.is_symlink()
    assert entry.is_symlink() and entry.resolve() == command.resolve()
    assert package.commands == ["demo-cli"]


def test_script_command_stays_in_the_plugin_and_usr_bin_links_to_it(tmp_path):
    plugin = make_plugin(
        tmp_path,
        files={
            "bin/demo-cli": "#!/usr/bin/env python3\n",
            "lib/demo_core/__init__.py": "",
        },
        executable=("bin/demo-cli",),
    )

    lay_out(tmp_path, plugin)

    root = tmp_path / "root" / "demo"
    script = root / TREE / "demo/bin/demo-cli"
    command = root / "usr/bin/demo-cli"
    assert script.is_file() and not script.is_symlink()
    # Relative, so it resolves the same inside the package root and once installed.
    assert not os.readlink(command).startswith("/")
    assert command.resolve() == script.resolve()


def test_lifecycle_scripts_are_dropped_and_claude_hooks_kept(tmp_path):
    plugin = make_plugin(
        tmp_path,
        files={
            "hooks/install-binaries.sh": "#!/usr/bin/env bash\n",
            "hooks/hooks.json": "{}",
        },
        executable=("hooks/install-binaries.sh",),
    )

    lay_out(tmp_path, plugin)

    hooks = tmp_path / "root/demo" / TREE / "demo/hooks"
    assert sorted(p.name for p in hooks.iterdir()) == ["hooks.json"]


def test_hooks_directory_holding_only_lifecycle_scripts_is_not_shipped(tmp_path):
    plugin = make_plugin(
        tmp_path,
        files={"hooks/install-binaries.sh": "#!/usr/bin/env bash\n"},
        executable=("hooks/install-binaries.sh",),
    )

    lay_out(tmp_path, plugin)

    assert not (tmp_path / "root/demo" / TREE / "demo/hooks").exists()


def test_file_modes_are_normalized_for_every_user(tmp_path):
    plugin = make_plugin(tmp_path, files={"skills/demo/SKILL.md": "x", "lib/run": "x"})
    (plugin / "skills/demo/SKILL.md").chmod(0o600)
    (plugin / "lib/run").chmod(0o700)

    lay_out(tmp_path, plugin)

    tree = tmp_path / "root/demo" / TREE / "demo"
    assert stat.S_IMODE((tree / "skills/demo/SKILL.md").stat().st_mode) == 0o644
    assert stat.S_IMODE((tree / "lib/run").stat().st_mode) == 0o755


def test_package_without_elf_files_is_architecture_independent(tmp_path):
    plugin = make_plugin(tmp_path, files={"skills/demo/SKILL.md": "x"})

    assert lay_out(tmp_path, plugin).architecture == "all"


def test_x86_64_binaries_make_an_amd64_package(tmp_path):
    plugin = make_plugin(
        tmp_path,
        files={"components/demo/demo-cgi": elf()},
        executable=("components/demo/demo-cgi",),
    )

    package = lay_out(tmp_path, plugin)

    assert package.architecture == "amd64"
    assert [p.name for p in package.elf_files] == ["demo-cgi"]


def test_binaries_for_two_architectures_are_rejected(tmp_path):
    plugin = make_plugin(
        tmp_path,
        files={"bin/a": elf(0x3E), "bin/b": elf(0xB7)},
        executable=("bin/a", "bin/b"),
    )

    with pytest.raises(PackagingError, match="more than one architecture"):
        lay_out(tmp_path, plugin)


def test_binary_for_an_unsupported_architecture_is_rejected(tmp_path):
    plugin = make_plugin(tmp_path, files={"bin/a": elf(0x28)}, executable=("bin/a",))

    with pytest.raises(PackagingError, match="unsupported ELF machine 0x28"):
        lay_out(tmp_path, plugin)


def test_package_names_carry_the_codexis_prefix_once():
    assert debpkg.package_name("ares") == "codexis-ares"
    assert debpkg.package_name("codexis") == "codexis"
    assert debpkg.package_name("codexis-dane") == "codexis-dane"


def test_depends_join_interpreters_plugin_dependencies_and_declared_tools(tmp_path):
    plugin = make_plugin(
        tmp_path,
        files={
            "bin/demo-cli": "#!/usr/bin/env python3\n",
            "components/demo/demo-cgi": '#!/usr/bin/env node\nrequire("node:fs")\n',
            "shared/run.sh": "#!/usr/bin/env bash\n",
        },
        executable=("bin/demo-cli", "components/demo/demo-cgi", "shared/run.sh"),
        dependencies=["codexis", "ares"],
    )
    meta_dir = meta(tmp_path, control="Depends: curl, jq\n")

    package = lay_out(tmp_path, plugin, meta_dir)

    assert package.depends == [
        "nodejs",
        "python3",
        "codexis",
        "codexis-ares",
        "curl",
        "jq",
    ]


def test_script_with_an_unmapped_interpreter_is_rejected(tmp_path):
    plugin = make_plugin(
        tmp_path,
        files={"bin/demo-cli": "#!/usr/bin/ruby\n"},
        executable=("bin/demo-cli",),
    )

    with pytest.raises(PackagingError, match="interpreter 'ruby'"):
        lay_out(tmp_path, plugin)


def test_node_bundle_loading_a_module_it_does_not_contain_is_rejected(tmp_path):
    bundle = '#!/usr/bin/env node\nrequire("node:fs");\nrequire("left-pad");\n'
    plugin = make_plugin(
        tmp_path, files={"bin/demo-cli": bundle}, executable=("bin/demo-cli",)
    )

    with pytest.raises(PackagingError, match="left-pad"):
        lay_out(tmp_path, plugin)


def test_declared_link_into_usr_bin_turns_a_data_script_into_a_command(tmp_path):
    plugin = make_plugin(
        tmp_path, files={"shared/scripts/render.sh": "#!/usr/bin/env bash\n"}
    )
    meta_dir = meta(
        tmp_path, links=f"{TREE}/demo/shared/scripts/render.sh usr/bin/demo-render\n"
    )

    package = lay_out(tmp_path, plugin, meta_dir)

    command = tmp_path / "root/demo/usr/bin/demo-render"
    assert os.access(command.resolve(), os.X_OK)
    assert package.commands == ["demo-render"]


def test_declared_link_to_a_file_the_plugin_does_not_ship_is_rejected(tmp_path):
    plugin = make_plugin(tmp_path)
    meta_dir = meta(tmp_path, links=f"{TREE}/demo/missing.sh usr/bin/demo-missing\n")

    with pytest.raises(PackagingError, match="not packaged"):
        lay_out(tmp_path, plugin, meta_dir)


def wheel(base: Path) -> tuple[str, str]:
    """A pure-Python wheel on disk: its file:// URL and sha256."""
    path = base / "demolib-1.0-py3-none-any.whl"
    with zipfile.ZipFile(path, "w") as archive:
        archive.writestr("demolib/__init__.py", "VALUE = 42\n")
        archive.writestr("demolib-1.0.dist-info/LICENSE", "BSD\n")
        archive.writestr("demolib-1.0.data/scripts/tool.py", "")
    return path.as_uri(), hashlib.sha256(path.read_bytes()).hexdigest()


def test_wheel_is_unpacked_into_the_plugin_lib(tmp_path):
    url, digest = wheel(tmp_path)
    plugin = make_plugin(tmp_path)
    meta_dir = meta(tmp_path, wheels=f"# vendored\n{url} sha256={digest}\n")

    package = lay_out(tmp_path, plugin, meta_dir)

    lib = tmp_path / "root/demo" / TREE / "demo/lib"
    assert sorted(p.name for p in lib.iterdir()) == ["demolib", "demolib-1.0.dist-info"]
    assert package.python_modules


def test_wheel_whose_hash_does_not_match_is_rejected(tmp_path):
    url, _ = wheel(tmp_path)
    plugin = make_plugin(tmp_path)
    meta_dir = meta(tmp_path, wheels=f"{url} sha256={'0' * 64}\n")

    with pytest.raises(PackagingError, match="sha256 mismatch"):
        lay_out(tmp_path, plugin, meta_dir)


def test_wheel_with_compiled_code_is_rejected(tmp_path):
    meta_dir = meta(
        tmp_path, wheels="https://x/numpy-2.0-cp312-cp312-linux_x86_64.whl sha256=ab\n"
    )

    with pytest.raises(PackagingError, match="only pure-Python wheels"):
        debpkg.read_wheels(meta_dir)


def test_depends_field_may_be_folded_and_commented(tmp_path):
    meta_dir = meta(
        tmp_path,
        control="# tools the binary spawns\nDepends: curl,\n  libreoffice-core | libreoffice\n",
    )

    assert debpkg.read_depends(meta_dir) == ["curl", "libreoffice-core | libreoffice"]


def test_control_accepts_only_depends(tmp_path):
    meta_dir = meta(tmp_path, control="Depends: curl\nRecommends: jq\n")

    with pytest.raises(PackagingError, match="expected a single Depends field"):
        debpkg.read_depends(meta_dir)


def test_control_carries_version_revision_maintainer_and_wrapped_description(tmp_path):
    plugin = make_plugin(
        tmp_path, description="A sentence long enough to need wrapping. " * 4
    )
    package = lay_out(tmp_path, plugin)
    package.shlibs = ["libc6 (>= 2.34)"]

    text = debpkg.control(package, 12)

    lines = text.splitlines()
    assert "Package: codexis-demo" in lines
    assert "Version: 1.2.3-1" in lines
    assert "Maintainer: ATLAS GROUP <dev@example.com>" in lines
    assert "Depends: libc6 (>= 2.34)" in lines
    assert "Description: Demo - Packaging" in lines
    extended = lines[lines.index("Description: Demo - Packaging") + 1 :]
    assert len(extended) > 1
    assert all(line.startswith(" ") and len(line) <= 79 for line in extended)


def marketplace(*names: str) -> dict:
    return {
        "owner": {"name": "ATLAS GROUP", "email": "dev@example.com"},
        "i18n": {"en": {"displayName": "Marketplace", "description": "All of it."}},
        "plugins": [{"name": name} for name in names],
    }


def test_marketplace_package_depends_on_every_listed_plugin(tmp_path):
    packages = [
        lay_out(tmp_path, make_plugin(tmp_path, name=name))
        for name in ("sid", "codexis")
    ]

    meta_package = debpkg.marketplace_package(
        marketplace("codexis", "sid"), packages, epoch=1791458640
    )

    assert meta_package.name == "codexis-marketplace"
    assert meta_package.depends == ["codexis (>= 1.2.3-1)", "codexis-sid (>= 1.2.3-1)"]
    assert meta_package.version == "20261008.112400"
    assert meta_package.architecture == "all"


def test_marketplace_listing_a_plugin_that_was_not_built_is_rejected(tmp_path):
    packages = [lay_out(tmp_path, make_plugin(tmp_path, name="sid"))]

    with pytest.raises(PackagingError, match="unbuilt plugins: ares"):
        debpkg.marketplace_package(marketplace("ares", "sid"), packages, epoch=0)


def test_marketplace_package_is_only_control_metadata(tmp_path):
    packages = [lay_out(tmp_path, make_plugin(tmp_path, name="sid"))]
    meta_package = debpkg.marketplace_package(marketplace("sid"), packages, epoch=0)
    root = tmp_path / "meta-root"
    root.mkdir()

    debpkg.write_metadata(meta_package, root)

    assert [p.name for p in (root / "DEBIAN").iterdir()] == ["control"]
    assert meta_package.filename == "codexis-marketplace_19700101.000000_all.deb"


def run_script(package: debpkg.Package, name: str, root: Path) -> None:
    script = debpkg.maintainer_scripts(package, PurePosixPath(root))[name]
    subprocess.run(["sh", "-c", script], check=True)


def installed(root: Path, *names: str) -> None:
    for name in names:
        (root / name / ".claude-plugin").mkdir(parents=True)
        (root / name / ".claude-plugin" / "plugin.json").write_text("{}")


def test_postinst_lists_every_installed_plugin_in_plugins_txt(tmp_path):
    package = lay_out(tmp_path, make_plugin(tmp_path))
    root = tmp_path / "installed"
    installed(root, "demo", "ares")
    (root / "not-a-plugin").mkdir()

    run_script(package, "postinst", root)

    assert (root / "plugins.txt").read_text().splitlines() == [
        str(root / "ares"),
        str(root / "demo"),
    ]


def test_removing_a_plugin_drops_it_from_plugins_txt(tmp_path):
    package = lay_out(tmp_path, make_plugin(tmp_path))
    root = tmp_path / "installed"
    installed(root, "demo", "ares")
    run_script(package, "postinst", root)

    run_script(package, "prerm", root)
    (root / "demo" / ".claude-plugin" / "plugin.json").unlink()
    run_script(package, "postrm", root)

    assert (root / "plugins.txt").read_text().splitlines() == [str(root / "ares")]


def test_removing_the_last_plugin_leaves_no_plugins_txt(tmp_path):
    package = lay_out(tmp_path, make_plugin(tmp_path))
    root = tmp_path / "installed"
    installed(root, "demo")
    run_script(package, "postinst", root)

    run_script(package, "prerm", root)

    assert list(root.glob("plugins.txt*")) == []


def test_prerm_cleans_python_caches_only_for_plugins_shipping_modules(tmp_path):
    with_modules = lay_out(tmp_path, make_plugin(tmp_path, files={"lib/core/x.py": ""}))
    without = lay_out(tmp_path, make_plugin(tmp_path, name="plain"))

    assert "__pycache__" in debpkg.maintainer_scripts(with_modules)["prerm"]
    assert "__pycache__" not in debpkg.maintainer_scripts(without)["prerm"]


def test_md5sums_list_payload_files_but_not_symlinks(tmp_path):
    plugin = make_plugin(
        tmp_path, files={"bin/demo-cli": elf()}, executable=("bin/demo-cli",)
    )
    package = lay_out(tmp_path, plugin)

    debpkg.write_metadata(package, tmp_path / "root/demo")

    sums = (tmp_path / "root/demo/DEBIAN/md5sums").read_text().splitlines()
    listed = sorted(line.split("  ", 1)[1] for line in sums)
    assert listed == ["usr/bin/demo-cli", f"{TREE}/demo/.claude-plugin/plugin.json"]


def build(tmp_path: Path, *plugins: str) -> int:
    out = tmp_path / "out"
    out.mkdir(exist_ok=True)
    return debpkg.main(
        [
            "--payload",
            str(tmp_path / "payload"),
            "--repo",
            str(tmp_path / "repo"),
            "--out",
            str(out),
            *plugins,
        ]
    )


def test_main_rejects_a_command_shipped_by_two_plugins(tmp_path, capsys):
    for name in ("one", "two"):
        make_plugin(
            tmp_path,
            name=name,
            files={"bin/same": "#!/bin/sh\n"},
            executable=("bin/same",),
        )

    assert build(tmp_path) == 1
    assert "/usr/bin/same is shipped by both" in capsys.readouterr().err
    assert not any((tmp_path / "out").iterdir())


def test_main_rejects_two_plugins_sharing_a_package_name(tmp_path, capsys):
    make_plugin(tmp_path, name="dane")
    make_plugin(tmp_path, name="codexis-dane")

    assert build(tmp_path) == 1
    assert "package name codexis-dane is taken twice" in capsys.readouterr().err


def test_main_rejects_an_unknown_plugin_name(tmp_path, capsys):
    make_plugin(tmp_path)

    assert build(tmp_path, "nope") == 1
    assert "unknown plugins: nope" in capsys.readouterr().err
