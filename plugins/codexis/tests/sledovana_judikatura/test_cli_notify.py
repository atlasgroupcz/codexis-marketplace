"""CLI notify wiring: notify show/set/recipients + save-report → notify gate.

Loads bin/cdx-sledovana-judikatura in-process (same harness test_cli_idempotent.py
uses) so module-level `notify`/`notify_settings` references can be monkeypatched
directly, and so its cmd_* functions run against the `judikatura_app_dir` fixture's
isolated state instead of touching the real ~/.cdx tree.
"""

import argparse
import importlib.machinery
import importlib.util
import json
import os
from unittest.mock import patch

import pytest

from sledovana_judikatura_core import tracking


def _load_cli():
    path = os.path.join(os.path.dirname(__file__), "..", "..", "bin", "cdx-sledovana-judikatura")
    loader = importlib.machinery.SourceFileLoader("cdx_sledovana_judikatura_notify", path)
    spec = importlib.util.spec_from_file_location(
        "cdx_sledovana_judikatura_notify", path, loader=loader
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_cli = _load_cli()  # defs-only/stateless — load once, reuse across tests


# ── notify show/set ─────────────────────────────────────────────────────────


def test_notify_show_defaults(judikatura_app_dir, capsys):
    _cli.cmd_notify_show(argparse.Namespace())
    out = json.loads(capsys.readouterr().out)
    assert out == {"email": False, "inApp": True, "recipients": []}


def test_notify_set_email(judikatura_app_dir, capsys):
    _cli.cmd_notify_set(argparse.Namespace(email=True, inapp=None))
    err = capsys.readouterr().err
    assert "e-mail: True" in err
    on_disk = json.loads((judikatura_app_dir / "settings.json").read_text())
    assert on_disk["notify"] == {"email": True}


# ── notify recipients ────────────────────────────────────────────────────────


def _recipients(app_dir):
    data = json.loads((app_dir / "settings.json").read_text())
    return data["notify"]["recipients"]


def test_recipients_add_and_remove(judikatura_app_dir, capsys):
    _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="a@x.cz"))
    _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="b@y.cz"))
    assert _recipients(judikatura_app_dir) == ["a@x.cz", "b@y.cz"]
    _cli.cmd_notify_recipients(argparse.Namespace(rcmd="remove", email="A@X.CZ"))
    assert _recipients(judikatura_app_dir) == ["b@y.cz"]


def test_recipients_set_and_clear(judikatura_app_dir):
    _cli.cmd_notify_recipients(argparse.Namespace(rcmd="set", emails=["a@x.cz", "b@y.cz"]))
    assert _recipients(judikatura_app_dir) == ["a@x.cz", "b@y.cz"]
    _cli.cmd_notify_recipients(argparse.Namespace(rcmd="clear", emails=[]))
    assert _recipients(judikatura_app_dir) == []


def test_recipients_add_invalid_errors(judikatura_app_dir, capsys):
    with pytest.raises(SystemExit) as exc:
        _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="nope"))
    assert exc.value.code == 1
    assert "Neplatná" in capsys.readouterr().err


# ── self-guard: adding your own address is a no-op (you always receive it) ────
#
# Driven in-process (not via subprocess) because dropping self needs
# notify._query_me_email mocked, which a subprocess can't reach.


def test_recipients_add_drops_signed_in_user(judikatura_app_dir):
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="ME@firma.cz"))
    assert _recipients(judikatura_app_dir) == []


def test_recipients_set_drops_signed_in_user_keeps_others(judikatura_app_dir):
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(
            argparse.Namespace(rcmd="set", emails=["me@firma.cz", "a@x.cz"])
        )
    assert _recipients(judikatura_app_dir) == ["a@x.cz"]


# Dropping the caller's address without a word reads as a silent failure — an agent
# that asked for it goes digging (`notify show`, `--help`) to find out what happened.


def test_recipients_add_self_says_it_was_dropped(judikatura_app_dir, capsys):
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="ME@firma.cz"))
    err = capsys.readouterr().err
    assert "ME@firma.cz vynechán" in err
    assert "přihlášeného uživatele" in err


def test_recipients_add_other_address_stays_quiet(judikatura_app_dir, capsys):
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"):
        _cli.cmd_notify_recipients(argparse.Namespace(rcmd="add", email="a@x.cz"))
    assert "vynechán" not in capsys.readouterr().err
    assert _recipients(judikatura_app_dir) == ["a@x.cz"]


# ── save-report → notify wiring ─────────────────────────────────────────────
#
# This can't be driven via subprocess: verifying the composed `to` value
# requires mocking notify._query_me_email and notify.email_change_summary, and
# unittest.mock.patch can't reach into a separate process.


def _report(found_count=1, **extra):
    return {
        "found_count": found_count,
        "period_from": "2026-01-01",
        "period_to": "2026-01-31",
        **extra,
    }


def test_save_report_forwards_resolved_to_with_signed_in_user(
    judikatura_app_dir, tmp_path, monkeypatch, capsys
):
    u, _ = tracking.init("Téma")
    report_path = tmp_path / "report.json"
    report_path.write_text(json.dumps(_report()))
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify",
        lambda: {"email": True, "inApp": False, "recipients": ["a@x.cz"]},
    )
    with patch.object(_cli.notify, "_query_me_email", return_value="me@firma.cz"), \
         patch.object(_cli.notify, "email_change_summary", return_value=0) as m:
        _cli.cmd_save_report(argparse.Namespace(uuid=u, file=str(report_path)))
    assert m.call_args.kwargs.get("to") == ["me@firma.cz", "a@x.cz"]


def test_save_report_forwards_none_when_no_recipients(
    judikatura_app_dir, tmp_path, monkeypatch, capsys
):
    u, _ = tracking.init("Téma")
    report_path = tmp_path / "report.json"
    report_path.write_text(json.dumps(_report()))
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify",
        lambda: {"email": True, "inApp": False, "recipients": []},
    )
    with patch.object(_cli.notify, "_query_me_email") as me_mock, \
         patch.object(_cli.notify, "email_change_summary", return_value=0) as m:
        _cli.cmd_save_report(argparse.Namespace(uuid=u, file=str(report_path)))
    assert m.call_args.kwargs.get("to") is None
    me_mock.assert_not_called()  # no extras → resolve_to short-circuits, no GraphQL call


def test_save_report_notify_off_sends_nothing(judikatura_app_dir, tmp_path, monkeypatch, capsys):
    u, _ = tracking.init("Téma")
    report_path = tmp_path / "report.json"
    report_path.write_text(json.dumps(_report()))
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify",
        lambda: {"email": False, "inApp": False, "recipients": []},
    )
    with patch.object(_cli.notify, "email_change_summary") as m_email, \
         patch.object(_cli.notify, "inapp_change_summary") as m_inapp:
        _cli.cmd_save_report(argparse.Namespace(uuid=u, file=str(report_path)))
    m_email.assert_not_called()
    m_inapp.assert_not_called()


def test_save_report_inapp_fires_bell(judikatura_app_dir, tmp_path, monkeypatch, capsys):
    u, _ = tracking.init("Téma")
    report_path = tmp_path / "report.json"
    report_path.write_text(json.dumps(_report()))
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify",
        lambda: {"email": False, "inApp": True, "recipients": []},
    )
    with patch.object(_cli.notify, "inapp_change_summary", return_value=True) as m:
        _cli.cmd_save_report(argparse.Namespace(uuid=u, file=str(report_path)))
    m.assert_called_once()
    assert "zvoneček" in capsys.readouterr().err


def test_save_report_id_stays_clean_on_stdout(judikatura_app_dir, tmp_path, monkeypatch, capsys):
    # Notification wiring must never leak onto stdout — only report_id belongs there.
    u, _ = tracking.init("Téma")
    report_path = tmp_path / "report.json"
    report_path.write_text(json.dumps(_report(report_id="2026-04-20")))
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify",
        lambda: {"email": True, "inApp": True, "recipients": []},
    )
    with patch.object(_cli.notify, "email_change_summary", return_value=1), \
         patch.object(_cli.notify, "inapp_change_summary", return_value=True):
        _cli.cmd_save_report(argparse.Namespace(uuid=u, file=str(report_path)))
    out, err = capsys.readouterr()
    assert out.strip() == "2026-04-20"
    assert "OK: report '2026-04-20' uložen" in err
