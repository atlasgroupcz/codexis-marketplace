"""Every marketplace plugin must own an install contract (expected.json) next to
its code, and that contract must name exactly the skills the plugin ships - so
"a plugin ships without an install spec" and "a new skill is never install-tested"
are both a red build rather than something to remember. A behavioral (promptfoo)
config is recommended but deliberately not enforced here: the eval tier is opt-in
in CI, so requiring a config that does not routinely run would add friction
without a signal (see CLAUDE.md)."""

from __future__ import annotations

import json
import re
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


# --- example-prompts.json -----------------------------------------------------
# Optional per-plugin file holding ready-made prompts the daemon renders on the
# plugin page. The rules live in a pure function so they can be exercised against
# in-memory payloads; the plugin-level test below just feeds it real files.
_EXAMPLE_PROMPT_LANGS = ("cs", "en", "sk")
_EXAMPLE_PROMPT_ID_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")


def _example_prompt_errors(data: object) -> list[str]:
    """Schema violations in an example-prompts.json payload; empty means valid.

    Returns every problem rather than raising on the first, so an author fixing a
    hand-written file sees the whole list in one run instead of one per attempt.
    """
    if not isinstance(data, dict):
        return ["top level must be a JSON object"]

    prompts = data.get("prompts")
    if not isinstance(prompts, list) or not prompts:
        return ["`prompts` must be a non-empty array"]

    errors: list[str] = []
    seen: set[str] = set()

    for index, item in enumerate(prompts):
        where = f"prompts[{index}]"
        if not isinstance(item, dict):
            errors.append(f"{where}: must be an object")
            continue

        prompt_id = item.get("id")
        if not isinstance(prompt_id, str) or not _EXAMPLE_PROMPT_ID_RE.match(prompt_id):
            errors.append(f"{where}: `id` must be kebab-case, got {prompt_id!r}")
        elif prompt_id in seen:
            errors.append(f"{where}: duplicate id {prompt_id!r}")
        else:
            seen.add(prompt_id)

        i18n = item.get("i18n")
        if not isinstance(i18n, dict):
            errors.append(f"{where}: `i18n` must be an object")
            continue

        for lang in _EXAMPLE_PROMPT_LANGS:
            entry = i18n.get(lang)
            if not isinstance(entry, dict):
                errors.append(f"{where}.i18n.{lang}: missing or not an object")
                continue
            text = entry.get("prompt")
            if not isinstance(text, str) or not text.strip():
                errors.append(f"{where}.i18n.{lang}.prompt: must be a non-empty string")

    return errors


def _valid_example_prompt(prompt_id: str = "najdi-parcelu") -> dict:
    """A minimal payload entry the assertions below mutate one field at a time."""
    return {
        "id": prompt_id,
        "i18n": {
            "cs": {"prompt": "Najdi parcelu 123/4 v k. u. Nove Mesto."},
            "en": {"prompt": "Look up parcel 123/4 in the Nove Mesto area."},
            "sk": {"prompt": "Najdi parcelu 123/4 v k. u. Nove Mesto."},
        },
    }


def test_valid_example_prompts_payload_has_no_errors():
    assert _example_prompt_errors({"prompts": [_valid_example_prompt()]}) == []


def test_example_prompts_must_not_be_empty():
    assert _example_prompt_errors({"prompts": []})


def test_example_prompt_id_must_be_kebab_case():
    entry = _valid_example_prompt()
    entry["id"] = "Najdi_Parcelu"
    assert _example_prompt_errors({"prompts": [entry]})


def test_example_prompt_ids_must_be_unique_within_a_file():
    payload = {"prompts": [_valid_example_prompt("same"), _valid_example_prompt("same")]}
    assert _example_prompt_errors(payload)


def test_example_prompt_requires_every_language():
    entry = _valid_example_prompt()
    del entry["i18n"]["sk"]
    assert _example_prompt_errors({"prompts": [entry]})


def test_example_prompt_text_must_not_be_blank():
    entry = _valid_example_prompt()
    entry["i18n"]["en"]["prompt"] = "   "
    assert _example_prompt_errors({"prompts": [entry]})


@pytest.mark.parametrize("name", _PLUGINS)
def test_example_prompts_match_the_schema(name: str):
    """Optional file, pinned shape.

    ocr, video-analyze and visualization ship none on purpose: they run as part of
    another request, so there is no opening prompt to suggest. Absence is valid, so
    only the contents are pinned. Em-dashes are not checked here -
    test_repo_hygiene.py::test_no_em_or_en_dashes_in_source already sweeps every
    tracked .json, and a second check would report one mistake twice.
    """
    path = PLUGINS / name / "example-prompts.json"
    if not path.is_file():
        pytest.skip("no example-prompts.json - the file is optional")

    rel = path.relative_to(REPO)
    try:
        data = json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        pytest.fail(f"{rel}: invalid JSON - {exc}")

    errors = _example_prompt_errors(data)
    assert not errors, (
        f"{rel}: does not match the example-prompts schema:\n  " + "\n  ".join(errors)
    )
