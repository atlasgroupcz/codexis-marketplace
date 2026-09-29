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
import struct
from pathlib import Path

import pytest
import yaml

REPO = Path(__file__).resolve().parents[2]
PLUGINS = REPO / "plugins"
MARKETPLACE = REPO / ".claude-plugin" / "marketplace.json"

MONITOR_PLUGINS = {
    "codexis-bozp": ("Monitor - BOZP", "MONITOR_BOZP_AI"),
    "codexis-cirkev": ("Monitor - Církev", "MONITOR_CIRKEV_AI"),
    "codexis-dane": ("Monitor - Daně", "MONITOR_DANE_AI"),
    "codexis-dusevni-vlastnictvi": (
        "Monitor - Ochrana duševního vlastnictví",
        "MONITOR_DUSEVNI_VLASTNICTVI_AI",
    ),
    "codexis-gdpr": ("Monitor - Ochrana osobních údajů", "MONITOR_GDPR_AI"),
    "codexis-obecni-samosprava": (
        "Monitor - Obecní samospráva",
        "MONITOR_OBECNI_SAMOSPRAVA_AI",
    ),
    "codexis-personalistika": ("Monitor - Personalistika", "MONITOR_PERSONALISTIKA_AI"),
    "codexis-rekodifikace": ("Monitor - Rekodifikace", "MONITOR_REKODIFIKACE_AI"),
    "codexis-skolstvi": ("Monitor - Školství", "MONITOR_SKOLSTVI_AI"),
    "codexis-stavebnictvi": ("Monitor - Stavebnictví", "MONITOR_STAVEBNICTVI_AI"),
    "codexis-ucetnictvi": ("Monitor - Účetnictví", "MONITOR_UCETNICTVI_AI"),
    "codexis-verejne-zakazky": (
        "Monitor - Veřejné zakázky",
        "MONITOR_VEREJNE_ZAKAZKY_AI",
    ),
    "codexis-zivotni-prostredi": (
        "Monitor - Životní prostředí",
        "MONITOR_ZIVOTNI_PROSTREDI_AI",
    ),
}


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


def test_marketplace_has_exact_monitor_plugin_contracts():
    marketplace = json.loads(MARKETPLACE.read_text())
    marketplace_names = {entry["name"] for entry in marketplace["plugins"]}
    marketplace_uuids = [entry["uuid"] for entry in marketplace["plugins"]]
    manifests = {
        path.parents[1].name: json.loads(path.read_text())
        for path in PLUGINS.glob("*/.claude-plugin/plugin.json")
    }
    actual = {
        name: manifest["accessible-part"]
        for name, manifest in manifests.items()
        if str(manifest.get("accessible-part", "")).startswith("MONITOR_")
    }
    expected = {
        name: license_part for name, (_, license_part) in MONITOR_PLUGINS.items()
    }

    assert actual == expected

    assert len(marketplace_uuids) == len(set(marketplace_uuids))

    for name, (display_name, license_part) in MONITOR_PLUGINS.items():
        manifest = manifests[name]
        marketplace_entry = next(
            entry for entry in marketplace["plugins"] if entry["name"] == name
        )
        assert name in marketplace_names
        assert marketplace_entry["uuid"]
        assert marketplace_entry["source"] == f"./plugins/{name}"
        assert manifest["name"] == name
        assert manifest["version"] == "1.0.0"
        assert manifest["accessible-part"] == license_part
        assert manifest["dependencies"] == ["codexis"]
        assert manifest["i18n"]["cs"]["displayName"] == display_name
        assert (PLUGINS / name / manifest["iconPath"]).is_file()
        assert len(list((PLUGINS / name / "skills").glob("*/SKILL.md"))) == 1


@pytest.mark.parametrize("name", sorted(MONITOR_PLUGINS))
def test_monitor_paywall_contract(name: str):
    manifest_path = PLUGINS / name / ".claude-plugin" / "plugin.json"
    manifest = json.loads(manifest_path.read_text())
    paywall = manifest["paywall"]
    illustration_path = PLUGINS / name / paywall["illustrationPath"]
    expected_contact_url = (
        "https://codexis.ai/poptavka?produkt=monitor-" + name.removeprefix("codexis-")
    )

    assert paywall["illustrationPath"] == "assets/paywall.png"
    assert paywall["contactUrl"] == expected_contact_url
    assert illustration_path.is_file()

    header = illustration_path.read_bytes()[:24]
    assert header.startswith(b"\x89PNG\r\n\x1a\n")
    assert struct.unpack(">II", header[16:24]) == (768, 768)


@pytest.mark.parametrize("name", sorted(MONITOR_PLUGINS))
def test_monitor_skill_reference_paths_exist(name: str):
    skill_root = PLUGINS / name / "skills" / name
    for document in skill_root.rglob("*.md"):
        references = set(
            re.findall(
                r"`((?:references/)?[A-Za-z0-9][A-Za-z0-9._/-]*\.md)`",
                document.read_text(),
            )
        )
        for reference in references:
            target = (
                skill_root / reference
                if reference.startswith("references/") or reference == "SKILL.md"
                else document.parent / reference
            )
            assert target.is_file(), (
                f"{document.relative_to(skill_root)}: missing {reference}"
            )


@pytest.mark.parametrize(
    "template",
    sorted(PLUGINS.glob("*/automation-templates/*.md")),
    ids=lambda path: f"{path.parents[1].name}:{path.name}",
)
def test_automation_template_skills_belong_to_owning_plugin(template: Path):
    owner = template.parents[1].name
    for full_name in _frontmatter(template).get("skill-full-names", []):
        assert full_name in _skills_on_disk(owner)


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

    the marketplace tier iterates the contract and fails on skills it cannot
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

    video-analyze and visualization ship none on purpose: they run as part of
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
