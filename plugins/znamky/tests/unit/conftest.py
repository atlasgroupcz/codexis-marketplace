"""Shared fixtures for znamky unit tests.

Adds znamky_core to sys.path and isolates filesystem state per test by
monkeypatching tracking.APP_DIR to a tmp_path - no test touches the real
~/.cdx/app-state/znamky/ directory.
"""

import os
import sys

import pytest

_LIB_PATH = os.path.join(os.path.dirname(__file__), "..", "..", "lib")
if _LIB_PATH not in sys.path:
    sys.path.insert(0, os.path.abspath(_LIB_PATH))


@pytest.fixture
def app_dir(tmp_path, monkeypatch):
    """Redirect znamky state I/O to a fresh tmp directory."""
    from znamky_core import tracking

    target = tmp_path / "sledovane"
    target.mkdir(parents=True, exist_ok=True)
    monkeypatch.setattr(tracking, "APP_DIR", str(target))
    return target
