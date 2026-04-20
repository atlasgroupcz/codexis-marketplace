"""Shared fixtures for codexis unit tests.

Adds both core libs (sledovana_judikatura_core, sledovane_dokumenty_core) to
sys.path and isolates filesystem state per test by monkeypatching APP_DIR.
"""

import os
import sys

import pytest

_LIB_PATH = os.path.join(os.path.dirname(__file__), "..", "lib")
if _LIB_PATH not in sys.path:
    sys.path.insert(0, os.path.abspath(_LIB_PATH))


@pytest.fixture
def judikatura_app_dir(tmp_path, monkeypatch):
    """Redirect sledovana-judikatura state I/O to a tmp directory."""
    from sledovana_judikatura_core import state

    target = tmp_path / "sledovana-judikatura"
    target.mkdir(parents=True, exist_ok=True)
    monkeypatch.setattr(state, "APP_DIR", str(target))
    return target


@pytest.fixture
def dokumenty_app_dir(tmp_path, monkeypatch):
    """Redirect sledovane-dokumenty state I/O to a tmp directory.

    Also redirects GROUPS_PATH so group ops land in the same tmp.
    """
    from sledovane_dokumenty_core import state

    target = tmp_path / "sledovane-dokumenty"
    target.mkdir(parents=True, exist_ok=True)
    monkeypatch.setattr(state, "APP_DIR", str(target))
    monkeypatch.setattr(state, "GROUPS_PATH", str(target / "groups.json"))
    return target
