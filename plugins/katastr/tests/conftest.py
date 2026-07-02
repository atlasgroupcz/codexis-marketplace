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

# The project runs pytest with --import-mode=importlib, which doesn't add a
# test module's own directory to sys.path. Add it explicitly so test modules
# can do plain `from _helpers import ...` regardless of import mode.
_TESTS_DIR = os.path.dirname(os.path.abspath(__file__))
if _TESTS_DIR not in sys.path:
    sys.path.insert(0, _TESTS_DIR)

from katastr_core import tracking
from _helpers import _cuzk_record  # noqa: F401 (re-exported for convenience)


@pytest.fixture
def app_dir(tmp_path, monkeypatch):
    """Redirect katastr state I/O to a fresh tmp directory.

    Overrides katastr_core.tracking.APP_DIR so every filesystem operation in
    tracking.py (load/save/remove proceedings) lands under tmp_path/rizeni.
    """
    target = tmp_path / "rizeni"
    target.mkdir(parents=True, exist_ok=True)
    monkeypatch.setattr(tracking, "APP_DIR", str(target))
    return target


@pytest.fixture
def mock_cuzk(monkeypatch):
    """Mock fetch_from_cuzk AND fetch_by_id with configurable per-proceeding responses.

    Returns dict. Keys: "data" maps parsed-number tuple → response dict, and
    "raise_exc" allows simulating API errors.

    fetch_by_id resolves by scanning the same "data" dict for a record whose
    "id" field matches — so once a proceeding is seeded via `add()`, later
    `check_one()` calls (which fetch by the stored api_id) still find it. When
    more than one proceeding is seeded, give each `_cuzk_record()` a distinct
    `record_id`; otherwise fetch_by_id can't tell them apart.
    """
    config = {"data": {}, "raise_exc": None}

    def _fetch(parsed):
        if config["raise_exc"]:
            raise config["raise_exc"]
        key = (
            parsed["typ_rizeni"], parsed["poradove_cislo"],
            parsed["rok"], parsed["kod_pracoviste"],
        )
        return config["data"].get(key)

    def _fetch_by_id(api_id):
        if config["raise_exc"]:
            raise config["raise_exc"]
        for record in config["data"].values():
            if record and record.get("id") == api_id:
                return record
        return None

    monkeypatch.setattr(tracking, "fetch_from_cuzk", _fetch)
    monkeypatch.setattr(tracking, "fetch_by_id", _fetch_by_id)
    return config
