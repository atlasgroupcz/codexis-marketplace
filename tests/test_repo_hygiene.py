"""Repo-structure guards that run as ordinary pytest tests.

These assert on the real repository tree, not on fixtures - a broken invariant
here fails in seconds instead of in the deploy pipeline.
"""

from __future__ import annotations

import os
import stat
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
    return any(parts[i:i + 2] == (".claude", "worktrees") for i in range(len(parts)))


def _guarded_scripts() -> list[Path]:
    found: list[Path] = []
    for pattern in EXECUTABLE_GLOBS:
        found.extend(REPO_ROOT.glob(pattern))
    return sorted(p for p in found if not _in_nested_worktree(p.relative_to(REPO_ROOT)))


# Computed once at import: both the parametrize decorator and the found-anything
# guard need it, and a recursive glob over the repo (including .git) is not free.
_GUARDED_SCRIPTS = _guarded_scripts()


@pytest.mark.parametrize("script", _GUARDED_SCRIPTS, ids=lambda p: str(p.relative_to(REPO_ROOT)))
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
        p.parent.name for p in PLUGINS_DIR.glob("*/acceptance")
        if p.is_dir()
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
    ".rs", ".ts", ".tsx", ".js", ".jsx", ".md", ".json", ".yaml", ".yml",
    ".sh", ".py", ".css", ".html", ".toml",
}
# Generated artifacts and heavyweight trees that are never hand-edited. `assets`
# holds compiled FE bundles; `testdata` holds captured external fixtures.
_DASH_PRUNE_DIRS = {
    ".git", "node_modules", "target", "dist", ".venv", ".turbo", "assets", "testdata",
}


def _dash_offenders() -> list[str]:
    offenders: list[str] = []
    for root, dirs, files in os.walk(REPO_ROOT):
        dirs[:] = [d for d in dirs if d not in _DASH_PRUNE_DIRS]
        rel_root = Path(root).relative_to(REPO_ROOT)
        if _in_nested_worktree(rel_root):
            dirs[:] = []
            continue
        for name in files:
            # CLAUDE.md names the characters as examples of the rule; lockfiles are
            # generated.
            if name == "CLAUDE.md" or name.endswith(".lock") or name.endswith("-lock.json"):
                continue
            if Path(name).suffix.lower() not in _DASH_SUFFIXES:
                continue
            path = Path(root) / name
            try:
                text = path.read_text(encoding="utf-8")
            except (UnicodeDecodeError, OSError):
                continue
            if any(ch in text for ch in _DASH_CHARS):
                offenders.append(str(path.relative_to(REPO_ROOT)))
    return sorted(offenders)


def test_no_em_or_en_dashes_in_source() -> None:
    offenders = _dash_offenders()
    assert not offenders, (
        "em-dash or en-dash (U+2014 / U+2013) found in authored source; the repo "
        "uses plain hyphens only (see CLAUDE.md). Replace with a hyphen. Offenders:\n  "
        + "\n  ".join(offenders)
    )
