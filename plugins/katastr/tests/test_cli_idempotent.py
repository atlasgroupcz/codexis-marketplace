"""CLI-layer idempotency: add-already-tracked and remove-not-tracked exit OK."""

import argparse

from katastr_core import tracking
from _helpers import _cuzk_record, _load_cli

_cli = _load_cli()  # defs-only/stateless — load once, reuse across tests


def test_add_already_tracked_is_ok(app_dir, mock_cuzk, capsys):
    mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
    tracking.add("V-1/2026-701")
    _cli.cmd_tracking_add(argparse.Namespace(cislo="V-1/2026-701", label=""))
    assert "je již sledováno" in capsys.readouterr().err


def test_add_already_tracked_keeps_label_and_hints(app_dir, mock_cuzk, capsys):
    mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
    tracking.add("V-1/2026-701", label="Původní")
    _cli.cmd_tracking_add(argparse.Namespace(cislo="V-1/2026-701", label="Novákovi"))
    err = capsys.readouterr().err
    assert tracking.show("V-1/2026-701")["label"] == "Původní"   # unchanged
    assert "set-label" in err


def test_remove_not_tracked_is_ok(app_dir, capsys):
    _cli.cmd_tracking_remove(argparse.Namespace(cislo="V-9/2026-701"))
    assert "není sledováno" in capsys.readouterr().err


def _one_result():
    """A single non-empty check result — the loop output is stubbed, so only the
    non-empty shape matters (it must clear the `if not results` guard)."""
    return [{"cislo_rizeni": "V-1/2026-701", "ok": True, "change": None, "error": None}]


def test_check_email_setting_reports_no_change_no_email(monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking, "check_all", _one_result)
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify", lambda: {"email": True, "inApp": False}
    )
    captured = {}

    def fake(results, to=None):
        captured["called"] = True
        return 0  # no changes: nothing sent

    monkeypatch.setattr(_cli.notify, "email_change_summary", fake)
    _cli.cmd_tracking_check(argparse.Namespace(cislo=None))
    err = capsys.readouterr().err
    assert captured["called"] is True
    assert "Žádné nové změny — e-mail neodeslán." in err


def test_check_email_reports_change_count(monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking, "check_all", _one_result)
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify", lambda: {"email": True, "inApp": False}
    )
    monkeypatch.setattr(_cli.notify, "email_change_summary", lambda results, to=None: 2)
    _cli.cmd_tracking_check(argparse.Namespace(cislo=None))
    assert "2 řízení se změnou" in capsys.readouterr().err


def test_check_inapp_setting_fires_bell_on_change(monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking, "check_all", _one_result)
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify", lambda: {"email": False, "inApp": True}
    )
    monkeypatch.setattr(_cli.notify, "inapp_change_summary", lambda results: True)
    _cli.cmd_tracking_check(argparse.Namespace(cislo=None))
    assert "zvoneček" in capsys.readouterr().err


def test_check_notify_off_sends_nothing(monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking, "check_all", _one_result)
    monkeypatch.setattr(
        _cli.notify_settings, "get_notify", lambda: {"email": False, "inApp": False}
    )
    calls = []
    monkeypatch.setattr(_cli.notify, "email_change_summary", lambda results: calls.append("email"))
    monkeypatch.setattr(_cli.notify, "inapp_change_summary", lambda results: calls.append("inapp"))
    _cli.cmd_tracking_check(argparse.Namespace(cislo=None))
    assert calls == []
