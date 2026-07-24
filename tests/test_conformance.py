"""Every marketplace plugin must own an install contract (expected.json) next to
its code, so "a plugin ships without an install spec" is a red build, not a thing
to remember. A behavioral (promptfoo) config is recommended but deliberately not
enforced here: the eval tier is opt-in in CI, so requiring a config that does not
routinely run would add friction without a signal (see CLAUDE.md)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

REPO = Path(__file__).resolve().parent.parent
PLUGINS = REPO / "plugins"
MARKETPLACE = REPO / ".claude-plugin" / "marketplace.json"


def _marketplace_plugins() -> list[str]:
    data = json.loads(MARKETPLACE.read_text())
    return sorted(p["name"] for p in data.get("plugins", []))


_PLUGINS = _marketplace_plugins()


def test_marketplace_lists_plugins():
    # Guard against the guard matching nothing (e.g. a moved manifest).
    assert _PLUGINS, "marketplace.json lists no plugins - is the manifest path right?"


@pytest.mark.parametrize("name", _PLUGINS)
def test_plugin_has_install_contract(name: str):
    expected = PLUGINS / name / "tests" / "install" / "expected.json"
    assert expected.is_file(), (
        f"{name}: missing tests/install/expected.json - every plugin needs an "
        f"install contract (the skills it registers)."
    )
