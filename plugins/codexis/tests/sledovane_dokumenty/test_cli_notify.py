"""CLI-layer notify recipients management for cdx-sledovane-dokumenty."""

import json
import os
import subprocess
import sys
from pathlib import Path

BIN = Path(__file__).resolve().parents[2] / "bin" / "cdx-sledovane-dokumenty"
LIB = Path(__file__).resolve().parents[2] / "lib"


def _run(args, home):
    env = {
        **os.environ,
        "HOME": str(home),
        "CODEXIS_PUBLIC_USER_HOME": str(home),
        "PYTHONPATH": str(LIB),
    }
    return subprocess.run(
        [sys.executable, str(BIN), *args], env=env, capture_output=True, text=True
    )


def _recipients(home):
    data = json.loads(
        (home / ".cdx" / "apps" / "sledovane-dokumenty" / "settings.json").read_text()
    )
    return data["notify"]["recipients"]


def test_recipients_add_and_remove(tmp_path):
    _run(["notify", "recipients", "add", "a@x.cz"], tmp_path)
    _run(["notify", "recipients", "add", "b@y.cz"], tmp_path)
    assert _recipients(tmp_path) == ["a@x.cz", "b@y.cz"]
    _run(["notify", "recipients", "remove", "A@X.CZ"], tmp_path)
    assert _recipients(tmp_path) == ["b@y.cz"]


def test_recipients_set_and_clear(tmp_path):
    _run(["notify", "recipients", "set", "a@x.cz", "b@y.cz"], tmp_path)
    assert _recipients(tmp_path) == ["a@x.cz", "b@y.cz"]
    _run(["notify", "recipients", "clear"], tmp_path)
    assert _recipients(tmp_path) == []


def test_recipients_add_invalid_errors(tmp_path):
    result = _run(["notify", "recipients", "add", "nope"], tmp_path)
    assert result.returncode == 1
    assert "Neplatná" in result.stderr
