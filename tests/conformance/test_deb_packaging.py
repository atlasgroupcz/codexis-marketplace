"""Guards that keep every plugin packageable by shared/deb/build-debs.sh.

The package build is not part of the tiers, so a broken `deb/` file or a script
on an interpreter the builder cannot map would otherwise surface only when
someone builds the packages.
"""

from __future__ import annotations

import subprocess
from pathlib import Path, PurePosixPath

import pytest

from shared.deb import debpkg

REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGINS = sorted(
    p
    for p in (REPO_ROOT / "plugins").iterdir()
    if (p / ".claude-plugin" / "plugin.json").is_file()
)


@pytest.mark.parametrize("plugin", PLUGINS, ids=lambda p: p.name)
def test_deb_metadata_parses_and_links_reach_shipped_files(plugin: Path) -> None:
    debpkg.read_fragment(plugin)
    tree = debpkg.INSTALL_ROOT / plugin.name
    for target, _ in debpkg.read_links(plugin):
        assert target.is_relative_to(tree), f"{target} is outside /{tree}"
        assert (plugin / target.relative_to(tree)).is_file(), f"/{target} missing"


def test_every_shipped_script_runs_on_a_packaged_interpreter() -> None:
    tracked = subprocess.run(
        ["git", "-C", str(REPO_ROOT), "ls-files", "-s", "--", "plugins"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.splitlines()
    unknown = []
    for line in tracked:
        mode, _, _, rel = line.split(maxsplit=3)
        parts = PurePosixPath(rel).parts
        if mode != "100755" or {"src", "tests"} & set(parts):
            continue
        program = debpkg.interpreter(REPO_ROOT / rel)
        if program is not None and program not in debpkg.INTERPRETERS:
            unknown.append(f"{rel} ({program})")
    assert not unknown, (
        "add the interpreter to INTERPRETERS in shared/deb/debpkg.py: "
        + ", ".join(unknown)
    )
