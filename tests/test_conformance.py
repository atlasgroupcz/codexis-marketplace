"""Every marketplace plugin must own an install contract (expected.json) next to
its code, and that contract must name exactly the skills the plugin ships - so
"a plugin ships without an install spec" and "a new skill is never install-tested"
are both a red build rather than something to remember. A behavioral (promptfoo)
config is recommended but deliberately not enforced here: the eval tier is opt-in
in CI, so requiring a config that does not routinely run would add friction
without a signal (see CLAUDE.md)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest
import yaml

sys.path.insert(0, str(Path(__file__).resolve().parent))

REPO = Path(__file__).resolve().parent.parent
PLUGINS = REPO / "plugins"
MARKETPLACE = REPO / ".claude-plugin" / "marketplace.json"


def _marketplace_plugins() -> list[str]:
    data = json.loads(MARKETPLACE.read_text())
    return sorted(p["name"] for p in data.get("plugins", []))


_PLUGINS = _marketplace_plugins()


def _contract_path(name: str) -> Path:
    return PLUGINS / name / "tests" / "install" / "expected.json"


def _skill_dirs() -> list[Path]:
    return sorted(PLUGINS.glob("*/skills/*/SKILL.md"))


def _skill_name(skill_md: Path) -> str:
    """The name the skill declares in its SKILL.md frontmatter.

    Frontmatter carrying a `name` is mandatory for every skill (see CLAUDE.md), so
    a file without one is a broken skill - not a case to quietly guess a name for.
    Guessing (e.g. falling back to the directory) would let a skill the daemon
    cannot register pass the contract check below.

    Requires the `---` fence at the very first byte, which is how all skills in
    this repo are written. Whether the daemon would also tolerate a BOM or a blank
    line ahead of the fence is not knowable from here, so this is the repo's own
    convention rather than a restatement of the runtime's parser.
    """
    rel = skill_md.relative_to(REPO)
    text = skill_md.read_text()

    # pytest.fail, not assert: assertion introspection on `text` dumps the whole
    # SKILL.md into the failure output and buries the message.
    if not text.startswith("---"):
        pytest.fail(f"{rel}: must open with a `---` YAML frontmatter fence.")

    frontmatter, sep, _ = text[3:].partition("\n---")
    if not sep:
        pytest.fail(f"{rel}: YAML frontmatter is never closed.")

    name = (yaml.safe_load(frontmatter) or {}).get("name")
    if not name:
        pytest.fail(f"{rel}: frontmatter declares no `name`.")
    return str(name)


def _skills_on_disk(name: str) -> set[str]:
    """Qualified names (`<plugin>:<skill>`) of the skills the plugin ships."""
    skills_dir = PLUGINS / name / "skills"
    return {f"{name}:{_skill_name(md)}" for md in skills_dir.glob("*/SKILL.md")}


def test_marketplace_lists_plugins():
    # Guard against the guard matching nothing (e.g. a moved manifest).
    assert _PLUGINS, "marketplace.json lists no plugins - is the manifest path right?"


@pytest.mark.parametrize("name", _PLUGINS)
def test_plugin_has_install_contract(name: str):
    expected = _contract_path(name)
    assert expected.is_file(), (
        f"{name}: missing tests/install/expected.json - every plugin needs an "
        f"install contract (the skills it registers)."
    )


@pytest.mark.parametrize("name", _PLUGINS)
def test_install_contract_names_every_skill_on_disk(name: str):
    """The contract must be exact, not a subset.

    tests/test-marketplace.py iterates the contract and fails on skills it cannot
    find after install; it never compares back the other way. A skill the contract
    omits is therefore never install- or uninstall-tested at all, and nothing else
    in the suite notices - codexis-dane and codexis-ucetnictvi shipped untested
    for months that way.
    """
    contract = _contract_path(name)
    if not contract.is_file():
        pytest.skip("no install contract - test_plugin_has_install_contract covers it")

    declared = set(json.loads(contract.read_text()).get("skills", []))
    on_disk = _skills_on_disk(name)

    assert declared == on_disk, (
        f"{name}: tests/install/expected.json is out of sync with skills/.\n"
        f"  shipped but not in the contract (never install-tested): "
        f"{sorted(on_disk - declared) or 'none'}\n"
        f"  in the contract but not shipped: "
        f"{sorted(declared - on_disk) or 'none'}"
    )


@pytest.mark.parametrize("name", _PLUGINS)
def test_plugin_has_readme(name: str):
    """Every marketplace plugin must explain itself to the user.

    23 of 26 plugins shipped without a README, so installing one told you nothing
    about what it does. Asserting the file exists keeps that from coming back with
    the next plugin. It deliberately says nothing about the contents - pinning
    prose against manifests constrains how the README may be written and catches
    only cosmetic drift.
    """
    readme = PLUGINS / name / "README.md"
    assert readme.is_file(), (
        f"{name}: missing README.md - a user installing this plugin has no "
        f"plain-language description of what it does."
    )


@pytest.mark.parametrize(
    "skill_md",
    _skill_dirs(),
    ids=lambda p: f"{p.parents[2].name}:{p.parent.name}",
)
def test_skill_directory_matches_declared_name(skill_md: Path):
    """A skill's directory and its frontmatter `name` must agree.

    Install contracts are written as `<plugin>:<skill>`, and every skill today has
    the same string in both places. Pinning that removes the ambiguity: were they
    to diverge, the contract would describe a name other than the one shipped.
    """
    declared = _skill_name(skill_md)
    assert declared == skill_md.parent.name, (
        f"{skill_md.relative_to(REPO)}: frontmatter name {declared!r} does not "
        f"match its directory {skill_md.parent.name!r} - pick one."
    )
