"""CLI-layer idempotency: dup add-text and remove-not-tracked exit OK."""

import argparse
import importlib.machinery
import importlib.util
import os

from znamky_core import tracking


def _load_cli():
    path = os.path.join(os.path.dirname(__file__), "..", "bin", "znamky-cli")
    loader = importlib.machinery.SourceFileLoader("znamky_cli", path)
    spec = importlib.util.spec_from_file_location("znamky_cli", path, loader=loader)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_cli = _load_cli()


def test_add_text_already_tracked_is_ok(app_dir, monkeypatch, capsys):
    def _raise(*a, **k):
        raise tracking.MarkAlreadyTrackedError("dup")
    monkeypatch.setattr(_cli.tracking, "add_text", _raise)
    _cli.cmd_add_text(argparse.Namespace(
        text="CODEXIS", nice="9", territory="CZ", owner="", label=""))
    assert "je již sledována" in capsys.readouterr().err


def test_add_text_already_tracked_hints_on_label(app_dir, monkeypatch, capsys):
    def _raise(*a, **k):
        raise tracking.MarkAlreadyTrackedError("dup")
    monkeypatch.setattr(_cli.tracking, "add_text", _raise)
    _cli.cmd_add_text(argparse.Namespace(
        text="CODEXIS", nice="9", territory="CZ", owner="", label="Klient"))
    assert "set-label" in capsys.readouterr().err


def test_remove_not_tracked_is_ok(app_dir, capsys):
    _cli.cmd_remove(argparse.Namespace(id="00000000-0000-0000-0000-000000000000"))
    assert "nic k odebrání" in capsys.readouterr().err
