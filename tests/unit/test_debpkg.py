"""Unit tests for the Debian packaging of plugins (shared/deb/debpkg.py).

Each test lays a small fixture plugin out the way the builder does. Nothing here
calls dpkg-deb or dpkg-shlibdeps; shared/deb/smoke-test.sh covers the built
packages end to end.
"""

from __future__ import annotations

import json
import os
import stat
from pathlib import Path

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
    directory = base / "meta" / name
    (directory / "deb").mkdir(parents=True)
    for filename, content in files.items():
        (directory / "deb" / filename).write_text(content)
    return directory


def lay_out(base: Path, plugin: Path, meta_dir: Path | None = None) -> debpkg.Package:
    root = base / "root" / plugin.name
    root.mkdir(parents=True)
    return debpkg.lay_out(plugin, meta_dir or base / "meta" / plugin.name, root)


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


def test_depends_join_interpreters_plugin_dependencies_and_declared_tools(tmp_path):
    plugin = make_plugin(
        tmp_path,
        files={
            "bin/demo-cli": "#!/usr/bin/env python3\n",
            "components/demo/demo-cgi": '#!/usr/bin/env node\nrequire("node:fs")\n',
            "shared/run.sh": "#!/usr/bin/env bash\n",
        },
        executable=("bin/demo-cli", "components/demo/demo-cgi", "shared/run.sh"),
        dependencies=["codexis"],
    )
    meta_dir = meta(tmp_path, control="Depends: curl\nRecommends: jq\n")

    package = lay_out(tmp_path, plugin, meta_dir)

    assert package.depends == ["nodejs", "python3", "codexis-codexis", "curl"]
    assert package.recommends == ["jq"]


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


def test_fragment_reads_folded_relations_and_skips_comments(tmp_path):
    meta_dir = meta(
        tmp_path,
        control="# tools the binary spawns\nDepends: curl,\n  libreoffice-core | libreoffice\n",
    )

    assert debpkg.read_fragment(meta_dir) == {
        "Depends": ["curl", "libreoffice-core | libreoffice"]
    }


def test_fragment_rejects_fields_the_builder_derives(tmp_path):
    meta_dir = meta(tmp_path, control="Version: 9.9.9\n")

    with pytest.raises(PackagingError, match="expected one of"):
        debpkg.read_fragment(meta_dir)


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
    assert not any(line.startswith("Recommends:") for line in lines)
    extended = lines[lines.index("Description: Demo - Packaging") + 1 :]
    assert len(extended) > 1
    assert all(line.startswith(" ") and len(line) <= 79 for line in extended)


def test_prerm_cleans_python_caches_only_for_plugins_shipping_modules(tmp_path):
    with_modules = lay_out(tmp_path, make_plugin(tmp_path, files={"lib/core/x.py": ""}))
    without = lay_out(tmp_path, make_plugin(tmp_path, name="plain"))

    for package in (with_modules, without):
        debpkg.write_metadata(package, tmp_path / "root" / package.plugin)

    script = tmp_path / "root/demo/DEBIAN/prerm"
    assert f"find /{TREE}/demo -type d -name __pycache__" in script.read_text()
    assert not (tmp_path / "root/plain/DEBIAN/prerm").exists()


def test_md5sums_list_payload_files_but_not_symlinks(tmp_path):
    plugin = make_plugin(
        tmp_path, files={"bin/demo-cli": elf()}, executable=("bin/demo-cli",)
    )
    package = lay_out(tmp_path, plugin)

    debpkg.write_metadata(package, tmp_path / "root/demo")

    sums = (tmp_path / "root/demo/DEBIAN/md5sums").read_text().splitlines()
    listed = sorted(line.split("  ", 1)[1] for line in sums)
    assert listed == ["usr/bin/demo-cli", f"{TREE}/demo/.claude-plugin/plugin.json"]


def test_main_rejects_a_command_shipped_by_two_plugins(tmp_path, capsys):
    for name in ("one", "two"):
        make_plugin(
            tmp_path,
            name=name,
            files={"bin/same": "#!/bin/sh\n"},
            executable=("bin/same",),
        )
    out = tmp_path / "out"
    out.mkdir()

    status = debpkg.main(
        [
            "--payload",
            str(tmp_path / "payload"),
            "--meta",
            str(tmp_path / "meta"),
            "--out",
            str(out),
        ]
    )

    assert status == 1
    assert "/usr/bin/same is shipped by both" in capsys.readouterr().err
    assert not any(out.iterdir())


def test_main_rejects_an_unknown_plugin_name(tmp_path, capsys):
    make_plugin(tmp_path)

    status = debpkg.main(
        [
            "--payload",
            str(tmp_path / "payload"),
            "--meta",
            str(tmp_path / "meta"),
            "--out",
            str(tmp_path),
            "nope",
        ]
    )

    assert status == 1
    assert "unknown plugins: nope" in capsys.readouterr().err
