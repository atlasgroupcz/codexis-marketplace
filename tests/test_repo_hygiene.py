"""Repo-structure guards that run as ordinary pytest tests.

These assert on the real repository tree, not on fixtures - a broken invariant
here fails in seconds instead of in the deploy pipeline.
"""

from __future__ import annotations

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
