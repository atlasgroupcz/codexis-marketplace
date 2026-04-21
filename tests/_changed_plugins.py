"""Detect which plugins changed between two git refs."""

from __future__ import annotations

import subprocess
from pathlib import Path


CORE_TRIGGERS = frozenset([
    ".claude-plugin/marketplace.json",
    "tests/test-plugin-e2e.py",
    "tests/_daemon_client.py",
    "tests/_chat_runner.py",
    "tests/_assertions.py",
    "tests/_changed_plugins.py",
    "tests/_transcript.py",
    "pyproject.toml",
])


def _git_diff_files(base_ref: str, repo_root: Path) -> list[str]:
    result = subprocess.run(
        ["git", "diff", "--name-only", f"{base_ref}...HEAD"],
        cwd=str(repo_root),
        capture_output=True,
        text=True,
        check=True,
    )
    return [line for line in result.stdout.splitlines() if line]


def _all_plugins_with_e2e(repo_root: Path) -> list[str]:
    plugins_dir = repo_root / "plugins"
    if not plugins_dir.is_dir():
        return []
    return sorted(
        p.name for p in plugins_dir.iterdir()
        if p.is_dir() and (p / "acceptance" / "e2e").is_dir()
    )


def get_changed_plugins(base_ref: str, repo_root: Path) -> list[str]:
    """Return sorted list of plugin names affected by diff.

    Returns all plugins with acceptance/e2e/ if any CORE_TRIGGERS file changed.
    """
    files = _git_diff_files(base_ref, repo_root)
    plugins_dir = repo_root / "plugins"
    changed: set[str] = set()
    for path in files:
        if path in CORE_TRIGGERS:
            return _all_plugins_with_e2e(repo_root)
        if path.startswith("plugins/"):
            parts = path.split("/")
            if len(parts) >= 2:
                name = parts[1]
                if (plugins_dir / name).is_dir():
                    changed.add(name)
    return sorted(changed)
