"""The move to tests/{install,e2e}/ must leave discovery self-consistent."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _changed_plugins import _all_plugins_with_e2e

REPO = Path(__file__).resolve().parent.parent
PLUGINS = REPO / "plugins"


def _dirs_with(rel: str) -> list[str]:
    return sorted(p.name for p in PLUGINS.iterdir()
                  if p.is_dir() and (p / rel).is_dir())


def test_e2e_discovery_matches_new_layout():
    on_disk = _dirs_with("tests/e2e")
    assert on_disk, "no plugin has tests/e2e/ yet - the move has not happened"
    assert _all_plugins_with_e2e(REPO) == on_disk


def test_install_specs_moved():
    assert list(PLUGINS.glob("*/tests/install/expected.json"))
