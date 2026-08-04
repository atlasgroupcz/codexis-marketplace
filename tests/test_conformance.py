"""Every marketplace plugin must own an install contract (expected.json) next to
its code, and that contract must name exactly the skills the plugin ships - so
"a plugin ships without an install spec" and "a new skill is never install-tested"
are both a red build rather than something to remember. A behavioral (promptfoo)
config is recommended but deliberately not enforced here: the eval tier is opt-in
in CI, so requiring a config that does not routinely run would add friction
without a signal (see CLAUDE.md)."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
import yaml

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


def _frontmatter(skill_md: Path) -> dict:
    """The parsed YAML frontmatter of a SKILL.md.

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

    return yaml.safe_load(frontmatter) or {}


def _skill_name(skill_md: Path) -> str:
    """The name the skill declares in its SKILL.md frontmatter.

    Frontmatter carrying a `name` is mandatory for every skill (see CLAUDE.md), so
    a file without one is a broken skill - not a case to quietly guess a name for.
    Guessing (e.g. falling back to the directory) would let a skill the daemon
    cannot register pass the contract check below.
    """
    rel = skill_md.relative_to(REPO)
    name = _frontmatter(skill_md).get("name")
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


# --- example prompts ----------------------------------------------------------
# Ready-made prompts the daemon renders for a skill, held per language under
# `i18n.<lang>.examplePrompts` in SKILL.md. The rules live in a pure function so
# they can be exercised against in-memory frontmatter; the skill-level test below
# just feeds it real files.
_EXAMPLE_PROMPT_LANGS = ("cs", "en", "sk")
_MAX_EXAMPLE_PROMPTS = 3


def _example_prompt_errors(i18n: object) -> list[str]:
    """Schema violations in a SKILL.md `i18n` block; empty means valid.

    Returns every problem rather than raising on the first, so an author fixing a
    hand-written frontmatter sees the whole list in one run instead of one per
    attempt. A skill with no prompts at all is valid - the block is optional.
    """
    if i18n is None:
        return []
    if not isinstance(i18n, dict):
        return ["`i18n` must be a mapping"]

    errors: list[str] = []
    counts: dict[str, int] = {}

    for lang, entry in i18n.items():
        where = f"i18n.{lang}.examplePrompts"
        if not isinstance(entry, dict):
            errors.append(f"i18n.{lang}: must be a mapping")
            continue

        prompts = entry.get("examplePrompts")
        if prompts is None:
            continue
        if not isinstance(prompts, list) or not prompts:
            errors.append(f"{where}: must be a non-empty list of strings")
            continue

        if len(prompts) > _MAX_EXAMPLE_PROMPTS:
            errors.append(
                f"{where}: at most {_MAX_EXAMPLE_PROMPTS} prompts, got {len(prompts)}"
            )
        for index, text in enumerate(prompts):
            if not isinstance(text, str) or not text.strip():
                errors.append(f"{where}[{index}]: must be a non-blank string")

        counts[str(lang)] = len(prompts)

    if not counts:
        return errors

    missing = [lang for lang in _EXAMPLE_PROMPT_LANGS if lang not in counts]
    if missing:
        errors.append(
            f"i18n: example prompts exist, so {', '.join(_EXAMPLE_PROMPT_LANGS)} "
            f"must all define them - missing {', '.join(missing)}"
        )

    if len(set(counts.values())) > 1:
        listed = ", ".join(f"{lang}={n}" for lang, n in sorted(counts.items()))
        errors.append(
            f"i18n: every language must list the same number of prompts - {listed}"
        )

    return errors


def _valid_i18n(prompts: list[str] | None = None) -> dict:
    """A minimal frontmatter block the assertions below mutate one field at a time."""
    prompts = prompts or ["Najdi parcelu 123/4 v k. u. Nove Mesto."]
    return {lang: {"examplePrompts": list(prompts)} for lang in _EXAMPLE_PROMPT_LANGS}


def test_valid_example_prompts_block_has_no_errors():
    assert _example_prompt_errors(_valid_i18n()) == []


def test_skill_without_example_prompts_is_valid():
    assert _example_prompt_errors({"cs": {"displayName": "Katastr"}}) == []


def test_example_prompts_must_not_be_empty_when_present():
    i18n = _valid_i18n()
    i18n["en"]["examplePrompts"] = []
    assert _example_prompt_errors(i18n)


def test_at_most_three_example_prompts_per_language():
    assert _example_prompt_errors(_valid_i18n(["a", "b", "c", "d"]))


def test_example_prompts_require_every_language():
    i18n = _valid_i18n()
    del i18n["sk"]
    assert _example_prompt_errors(i18n)


def test_example_prompts_must_be_the_same_count_in_every_language():
    i18n = _valid_i18n(["a", "b"])
    i18n["en"]["examplePrompts"] = ["a"]
    assert _example_prompt_errors(i18n)


def test_example_prompt_text_must_not_be_blank():
    i18n = _valid_i18n()
    i18n["en"]["examplePrompts"] = ["   "]
    assert _example_prompt_errors(i18n)


@pytest.mark.parametrize(
    "skill_md",
    _skill_dirs(),
    ids=lambda p: f"{p.parents[2].name}:{p.parent.name}",
)
def test_example_prompts_match_the_schema(skill_md: Path):
    """Optional block, pinned shape.

    ocr, video-analyze and visualization ship none on purpose: they run as part of
    another request, so there is no opening prompt to suggest. Absence is valid, so
    only the contents are pinned. Em-dashes are not checked here -
    test_repo_hygiene.py already sweeps every tracked source file, and a second
    check would report one mistake twice.
    """
    errors = _example_prompt_errors(_frontmatter(skill_md).get("i18n"))
    assert not errors, (
        f"{skill_md.relative_to(REPO)}: example prompts do not match the schema:\n  "
        + "\n  ".join(errors)
    )
