"""Repo-structure guards that run as ordinary pytest tests.

These assert on the real repository tree, not on fixtures - a broken invariant
here fails in seconds instead of in the deploy pipeline.
"""

from __future__ import annotations

import stat
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent

# Shell scripts the deploy pipeline executes directly. A missing +x bit on any of
# these has broken the pipeline before, so guard the whole class of them.
EXECUTABLE_GLOBS = ("**/build.sh", "**/create-dist.sh")


def _in_nested_worktree(rel: Path) -> bool:
    # A worktree checkout lives at .claude/worktrees/<name>/... - match those two
    # path segments in order, not a raw substring (which would also strip an
    # unrelated dir like .claude/worktreesnapshot). rel is relative to REPO_ROOT,
    # which may itself be such a checkout.
    parts = rel.parts
    return any(parts[i : i + 2] == (".claude", "worktrees") for i in range(len(parts)))


def _guarded_scripts() -> list[Path]:
    found: list[Path] = []
    for pattern in EXECUTABLE_GLOBS:
        found.extend(REPO_ROOT.glob(pattern))
    return sorted(p for p in found if not _in_nested_worktree(p.relative_to(REPO_ROOT)))


# Computed once at import: both the parametrize decorator and the found-anything
# guard need it, and a recursive glob over the repo (including .git) is not free.
_GUARDED_SCRIPTS = _guarded_scripts()


@pytest.mark.parametrize(
    "script", _GUARDED_SCRIPTS, ids=lambda p: str(p.relative_to(REPO_ROOT))
)
def test_build_scripts_are_executable(script: Path) -> None:
    mode = script.stat().st_mode
    assert mode & stat.S_IXUSR, (
        f"{script.relative_to(REPO_ROOT)} is not executable (owner +x missing). "
        f"Fix: chmod +x {script.relative_to(REPO_ROOT)}"
    )


def test_guard_actually_found_scripts() -> None:
    # Guard against the guard silently matching nothing (e.g. a bad glob).
    assert _GUARDED_SCRIPTS, "no build.sh/create-dist.sh found - glob is wrong"


PLUGINS_DIR = REPO_ROOT / "plugins"


def test_no_plugin_keeps_a_legacy_acceptance_dir() -> None:
    """Data specs live under tests/{install,e2e}/ now; a top-level
    plugins/<name>/acceptance/ means a plugin was added on the old layout."""
    offenders = sorted(
        p.parent.name for p in PLUGINS_DIR.glob("*/acceptance") if p.is_dir()
    )
    assert not offenders, (
        f"plugins still using the legacy acceptance/ layout: {offenders}. "
        f"Move specs to tests/install/expected.json and tests/e2e/*.yaml."
    )


# CLAUDE.md rule: authored text uses plain hyphens only - never an em-dash
# (U+2014) or en-dash (U+2013). A one-time sweep regressed within hours because
# nothing enforced it and multiple authors kept reintroducing them, so guard it.
_DASH_CHARS = (chr(0x2014), chr(0x2013))  # em-dash, en-dash
# Hand-editable text where the rule applies.
_DASH_SUFFIXES = {
    ".rs",
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".md",
    ".json",
    ".yaml",
    ".yml",
    ".sh",
    ".py",
    ".css",
    ".html",
    ".toml",
}
# Compiled FE bundles (under `assets/`) and captured external fixtures (under
# `testdata/`) are committed but not hand-edited, so they are exempt.
_DASH_EXEMPT_DIRS = {"assets", "testdata"}


def _dash_offenders() -> list[str]:
    # Only tracked files: the guard enforces committed source. Using git ls-files
    # (rather than walking the tree) skips gitignored working dirs like
    # .superpowers, .venv, node_modules, dist and target for free.
    tracked = subprocess.run(
        ["git", "-C", str(REPO_ROOT), "ls-files", "-z"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split("\0")
    offenders: list[str] = []
    for rel in tracked:
        if not rel:
            continue
        path = REPO_ROOT / rel
        if path.suffix.lower() not in _DASH_SUFFIXES:
            continue
        if path.name.endswith(".lock") or path.name.endswith("-lock.json"):
            continue
        if _DASH_EXEMPT_DIRS.intersection(path.parts):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        if any(ch in text for ch in _DASH_CHARS):
            offenders.append(rel)
    return sorted(offenders)


def test_no_em_or_en_dashes_in_source() -> None:
    offenders = _dash_offenders()
    assert not offenders, (
        "em-dash or en-dash (U+2014 / U+2013) found in authored source; the repo "
        "uses plain hyphens only (see CLAUDE.md). Replace with a hyphen. Offenders:\n  "
        + "\n  ".join(offenders)
    )


# Plugin binaries are committed artifacts, and the guest VM that runs them is
# x86-64. The build used to take its target platform from the build machine's
# own CPU, so an arm64 host produced arm64 binaries: cargo succeeded, the
# artifacts were committed like any other, and nothing complained until the
# daemon tried to exec them. shared/scripts/plugin-build.sh now pins
# linux/amd64; this guards the outcome, which is what actually ships.
_ELF_MAGIC = b"\x7fELF"
_ELF_HEADER_LEN = 20  # through e_machine at offset 18..19
_ELF_LITTLE_ENDIAN = 1  # EI_DATA
_ELF_MACHINE_X86_64 = 0x3E
_ELF_MACHINE_NAMES = {0x3E: "x86-64", 0xB7: "aarch64", 0x28: "arm", 0x03: "i386"}


def _plugin_elf_binaries() -> list[tuple[str, int]]:
    """Every tracked ELF under plugins/, as (path, e_machine).

    Covers bin/ and the CGI binaries that sit in components/ alike, without
    naming either - a new location would otherwise escape the check silently.
    """
    tracked = subprocess.run(
        ["git", "-C", str(REPO_ROOT), "ls-files", "-z", "--", "plugins"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split("\0")
    found: list[tuple[str, int]] = []
    for rel in tracked:
        if not rel:
            continue
        try:
            with (REPO_ROOT / rel).open("rb") as handle:
                header = handle.read(_ELF_HEADER_LEN)
        except OSError:
            continue
        if not header.startswith(_ELF_MAGIC) or len(header) < _ELF_HEADER_LEN:
            continue
        if header[5] != _ELF_LITTLE_ENDIAN:
            found.append((rel, -1))  # big-endian: wrong by definition here
            continue
        found.append((rel, int.from_bytes(header[18:20], "little")))
    return sorted(found)


_PLUGIN_ELF_BINARIES = _plugin_elf_binaries()


def test_committed_plugin_binaries_are_x86_64() -> None:
    offenders = [
        f"{rel} ({_ELF_MACHINE_NAMES.get(machine, hex(machine))})"
        for rel, machine in _PLUGIN_ELF_BINARIES
        if machine != _ELF_MACHINE_X86_64
    ]
    assert not offenders, (
        "committed plugin binaries must be x86-64 - the guest VM runs nothing "
        "else, and a foreign build fails only at exec time. Rebuild with "
        "./build.sh, which pins linux/amd64. Offenders:\n  " + "\n  ".join(offenders)
    )


def test_binary_arch_guard_actually_found_binaries() -> None:
    # Without this, moving bin/ or dropping the artifacts turns the check above
    # into a test that passes because it inspects nothing.
    assert _PLUGIN_ELF_BINARIES, (
        "no ELF binaries found under plugins/ - guard is looking in the wrong place"
    )
