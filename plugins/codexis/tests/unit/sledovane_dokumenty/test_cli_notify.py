"""CLI-layer notify recipients management for cdx-sledovane-dokumenty."""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

from tests.harness.script_loader import load_script

BIN = Path(__file__).resolve().parents[3] / "bin" / "cdx-sledovane-dokumenty"
LIB = Path(__file__).resolve().parents[3] / "lib"


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
        (
            home / ".cdx" / "app-state" / "sledovane-dokumenty" / "settings.json"
        ).read_text()
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


# ── self-guard: adding your own address is a no-op (you always receive it) ────
#
# Driven in-process (not via _run) because dropping self needs
# notify._query_me_email mocked, which a subprocess can't reach.


def _load_cli():
    return load_script("cdx_sledovane_dokumenty_notify", str(BIN))


_cli = _load_cli()  # defs-only/stateless - load once, reuse across tests


def _recipients_dir(app_dir):
    return json.loads((app_dir / "settings.json").read_text())["notify"]["recipients"]


@pytest.fixture
def _cli_home(dokumenty_app_dir, monkeypatch, tmp_path):
    # dokumenty_app_dir redirects state.APP_DIR; pin HOME so settings.defaults_path()
    # (which reads the real ``~``) can't pick up dev-machine data.
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    return dokumenty_app_dir


def test_recipients_add_drops_signed_in_user(_cli_home):
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="ME@firma.cz"))
    assert _recipients_dir(_cli_home) == []


# Dropping the caller's address without a word reads as a silent failure - an agent
# that asked for it goes digging (`notify show`, `--help`) to find out what happened.


def test_recipients_add_self_says_it_was_dropped(_cli_home, capsys):
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="ME@firma.cz"))
    err = capsys.readouterr().err
    assert "ME@firma.cz vynechán" in err
    assert "přihlášeného uživatele" in err


def test_recipients_add_other_address_stays_quiet(_cli_home, capsys):
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="a@x.cz"))
    assert "vynechán" not in capsys.readouterr().err
    assert _recipients_dir(_cli_home) == ["a@x.cz"]


def test_recipients_set_drops_signed_in_user_keeps_others(_cli_home):
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(
            argparse.Namespace(rcmd="set", emails=["me@firma.cz", "a@x.cz"])
        )
    assert _recipients_dir(_cli_home) == ["a@x.cz"]


# ── a refused bell ends the run red ──────────────────────────────────────────


def _project_check_result(**overrides):
    base = {
        "checked": 1,
        "changes_found": 1,
        "errors": [],
        "notify_failed": False,
        "results": [],
    }
    return {**base, **overrides}


def test_project_check_ends_the_run_red_when_the_bell_is_refused(monkeypatch, capsys):
    # The changes are found and saved by this point; the user was simply never
    # told. A scheduled run reports only through its exit code, so this is the
    # one signal that says so.
    monkeypatch.setattr(
        _cli.project_check,
        "check_watches",
        lambda *a, **k: _project_check_result(notify_failed=True),
    )

    with pytest.raises(SystemExit) as exit_info:
        _cli.cmd_project_check(
            argparse.Namespace(project=None, folder=None, no_notify=False)
        )

    assert exit_info.value.code == 1
    assert "nepodařilo" in capsys.readouterr().err


def test_project_check_stays_green_for_the_check_s_own_errors(monkeypatch, capsys):
    # Errors the check itself collected keep exiting zero, exactly as before.
    # Turning those red is a separate decision and does not ride on this fix.
    monkeypatch.setattr(
        _cli.project_check,
        "check_watches",
        lambda *a, **k: _project_check_result(
            changes_found=0, errors=["projekt nedostupný"]
        ),
    )

    _cli.cmd_project_check(
        argparse.Namespace(project=None, folder=None, no_notify=False)
    )

    assert "nepodařilo" not in capsys.readouterr().err
