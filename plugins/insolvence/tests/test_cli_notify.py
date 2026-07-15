import argparse
import importlib.machinery
import importlib.util
import json, os, subprocess, sys
from pathlib import Path
from unittest.mock import patch

BIN = Path(__file__).resolve().parents[1] / "bin" / "insolvence-cli"

def _run(args, home):
    env = {**os.environ, "HOME": str(home), "CODEXIS_PUBLIC_USER_HOME": str(home),
           "PYTHONPATH": str(Path(__file__).resolve().parents[1] / "lib")}
    return subprocess.run([sys.executable, str(BIN), *args], env=env,
                          capture_output=True, text=True)

def test_notify_show_then_set(tmp_path):
    out = _run(["notify", "show"], tmp_path)
    assert out.returncode == 0 and '"email": false' in out.stdout
    set_out = _run(["notify", "set", "--email"], tmp_path)
    assert set_out.returncode == 0
    data = json.loads((tmp_path / ".cdx" / "apps" / "insolvence" / "settings.json").read_text())
    assert data["notify"]["email"] is True


def _recipients(home):
    data = json.loads((home / ".cdx" / "apps" / "insolvence" / "settings.json").read_text())
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
# notify._query_me_email mocked, which a subprocess can't reach — same reason
# the check-wiring tests below load the CLI in-process.


def test_recipients_add_drops_signed_in_user(monkeypatch, tmp_path):
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="ME@firma.cz"))
    assert _recipients(tmp_path) == []


def test_recipients_set_drops_signed_in_user_keeps_others(monkeypatch, tmp_path):
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(
            argparse.Namespace(rcmd="set", emails=["me@firma.cz", "a@x.cz"])
        )
    assert _recipients(tmp_path) == ["a@x.cz"]


# Dropping the caller's address without a word reads as a silent failure — an agent
# that asked for it goes digging (`notify show`, `--help`) to find out what happened.


def test_recipients_add_self_says_it_was_dropped(monkeypatch, tmp_path, capsys):
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="ME@firma.cz"))
    err = capsys.readouterr().err
    assert "ME@firma.cz vynechán" in err
    assert "přihlášeného uživatele" in err


def test_recipients_add_other_address_stays_quiet(monkeypatch, tmp_path, capsys):
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="a@x.cz"))
    assert "vynechán" not in capsys.readouterr().err
    assert _recipients(tmp_path) == ["a@x.cz"]


# ── check wiring: resolve_to() feeds email_change_summary(to=...) ────────────
#
# This can't be driven through the subprocess `_run()` helper above: verifying
# the composed `to` value requires monkeypatching notify._query_me_email and
# notify.email_change_summary. Load bin/insolvence-cli in-process (same
# pattern as test_cli_idempotent.py's `_load_cli()`) so its module-level
# `tracking`/`notify`/`notify_settings` references can be mocked directly.


def _load_cli():
    path = os.path.join(os.path.dirname(__file__), "..", "bin", "insolvence-cli")
    loader = importlib.machinery.SourceFileLoader("insolvence_cli", path)
    spec = importlib.util.spec_from_file_location("insolvence_cli", path, loader=loader)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_cli = _load_cli()  # defs-only/stateless — load once, reuse across tests


def _one_result():
    """A single non-empty check result — just needs to clear the CLI's
    `if not results` guard so the notify path is reached."""
    return [{"ok": True, "display_name": "Jan Novák", "changes": []}]


def test_check_forwards_resolved_to_with_signed_in_user(monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking, "check_all", _one_result)
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify",
        lambda: {"email": True, "inApp": False, "recipients": ["a@x.cz"]},
    )
    monkeypatch.setattr(_cli.notify, "_query_me_email", lambda: "me@firma.cz")
    calls = {}
    monkeypatch.setattr(
        _cli.notify, "email_change_summary",
        lambda results, **k: calls.update(k) or 0,
    )
    _cli.cmd_check(argparse.Namespace(id=None))
    assert calls.get("to") == ["me@firma.cz", "a@x.cz"]


def test_check_forwards_none_when_no_recipients(monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking, "check_all", _one_result)
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify",
        lambda: {"email": True, "inApp": False, "recipients": []},
    )
    me_calls = []
    monkeypatch.setattr(_cli.notify, "_query_me_email", lambda: me_calls.append(1))
    calls = {}
    monkeypatch.setattr(
        _cli.notify, "email_change_summary",
        lambda results, **k: calls.update(k) or 0,
    )
    _cli.cmd_check(argparse.Namespace(id=None))
    assert calls.get("to") is None
    assert me_calls == []  # no extras → resolve_to short-circuits, no GraphQL call
