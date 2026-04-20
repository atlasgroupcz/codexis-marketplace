"""Shared fixtures for katastr unit tests.

Adds katastr_core to sys.path and isolates filesystem state per test by
monkeypatching APP_DIR to a tmp_path — no tests ever touch the real
~/.cdx/apps/katastr/ directory.
"""

import os
import sys

import pytest

_LIB_PATH = os.path.join(os.path.dirname(__file__), "..", "lib")
if _LIB_PATH not in sys.path:
    sys.path.insert(0, os.path.abspath(_LIB_PATH))


@pytest.fixture
def app_dir(tmp_path, monkeypatch):
    """Redirect katastr state I/O to a fresh tmp directory.

    Overrides katastr_core.tracking.APP_DIR so every filesystem operation in
    tracking.py (load/save/remove proceedings) lands under tmp_path/rizeni.
    """
    from katastr_core import tracking

    target = tmp_path / "rizeni"
    target.mkdir(parents=True, exist_ok=True)
    monkeypatch.setattr(tracking, "APP_DIR", str(target))
    return target
