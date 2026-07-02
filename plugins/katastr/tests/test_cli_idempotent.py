"""CLI-layer idempotency: add-already-tracked and remove-not-tracked exit OK."""

import argparse
import importlib.machinery
import importlib.util
import os

import pytest

from katastr_core import tracking
from _helpers import _cuzk_record


def _load_cli():
    path = os.path.join(os.path.dirname(__file__), "..", "bin", "katastr-cli")
    # bin/katastr-cli has no .py suffix, so spec_from_file_location can't
    # auto-detect a loader from the extension — pass one explicitly.
    loader = importlib.machinery.SourceFileLoader("katastr_cli", path)
    spec = importlib.util.spec_from_file_location("katastr_cli", path, loader=loader)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)  # defs only; main() is guarded by __main__
    return module


_cli = _load_cli()  # defs-only/stateless — load once, reuse across tests


def test_add_already_tracked_is_ok(app_dir, mock_cuzk, capsys):
    mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
    tracking.add("V-1/2026-701")
    _cli.cmd_tracking_add(argparse.Namespace(cislo="V-1/2026-701", label=""))
    assert "je již sledováno" in capsys.readouterr().out


def test_add_already_tracked_updates_label(app_dir, mock_cuzk, capsys):
    mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
    tracking.add("V-1/2026-701")
    _cli.cmd_tracking_add(argparse.Namespace(cislo="V-1/2026-701", label="Novákovi"))
    assert tracking.show("V-1/2026-701")["label"] == "Novákovi"


def test_remove_not_tracked_is_ok(app_dir, capsys):
    _cli.cmd_tracking_remove(argparse.Namespace(cislo="V-9/2026-701"))
    assert "není sledováno" in capsys.readouterr().out


def _one_result():
    """A single non-empty check result — the loop output is stubbed, so only the
    non-empty shape matters (it must clear the `if not results` guard)."""
    return [{"cislo_rizeni": "V-1/2026-701", "ok": True, "change": None, "error": None}]


def test_check_email_forwards_to_and_reports_heartbeat(monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking, "check_all", _one_result)
    captured = {}

    def fake(results, to=None):
        captured["to"] = to
        return 0  # heartbeat: sent, no changes

    monkeypatch.setattr(_cli.notify, "email_change_summary", fake)
    _cli.cmd_tracking_check(argparse.Namespace(cislo=None, email=True, to=["x@y.cz"]))
    out = capsys.readouterr().out
    assert captured["to"] == ["x@y.cz"]
    assert "beze změn" in out


def test_check_email_reports_change_count(monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking, "check_all", _one_result)
    monkeypatch.setattr(_cli.notify, "email_change_summary", lambda results, to=None: 2)
    _cli.cmd_tracking_check(argparse.Namespace(cislo=None, email=True, to=None))
    assert "2 řízení se změnou" in capsys.readouterr().out
