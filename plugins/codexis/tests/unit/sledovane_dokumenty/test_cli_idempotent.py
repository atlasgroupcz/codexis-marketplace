"""CLI-layer idempotency for cdx-sledovane-dokumenty add/remove."""

import argparse
import os

from sledovane_dokumenty_core import state, tracking

from tests._script_loader import load_script


def _load_cli():
    path = os.path.join(
        os.path.dirname(__file__), "..", "..", "..", "bin", "cdx-sledovane-dokumenty"
    )
    return load_script("cdx_sledovane_dokumenty", path)


_cli = _load_cli()


def test_add_already_tracked_is_ok(dokumenty_app_dir, monkeypatch, capsys):
    def _raise(*a, **k):
        raise tracking.DocumentAlreadyTrackedError("dup")

    monkeypatch.setattr(_cli.tracking, "add", _raise)
    _cli.cmd_add(argparse.Namespace(codexis_id="ZAK1", parts=None))
    assert "je už sledován" in capsys.readouterr().err


def test_remove_not_tracked_is_ok(dokumenty_app_dir, monkeypatch, capsys):
    def _raise(*a, **k):
        raise tracking.DocumentNotTrackedError("nope")

    monkeypatch.setattr(_cli.tracking, "remove", _raise)
    _cli.cmd_remove(argparse.Namespace(codexis_id="ZAK1"))
    assert "není sledován" in capsys.readouterr().err


def test_group_remove_absent_is_ok(dokumenty_app_dir, monkeypatch, capsys):
    def _raise(*a, **k):
        raise tracking.GroupNotFoundError("no group")

    monkeypatch.setattr(_cli.tracking, "group_remove", _raise)
    _cli.cmd_group_remove(argparse.Namespace(codexis_id="ZAK1", name=["Skupina"]))
    assert "nic k odebrání" in capsys.readouterr().err


def test_related_remove_not_tracked_is_ok(dokumenty_app_dir, capsys):
    # empty app dir → state.load_state returns None → soft OK, no exit
    _cli.cmd_related_remove(argparse.Namespace(codexis_id="ZAK1", type="amendments"))
    assert "nic k odebrání" in capsys.readouterr().err


def test_group_remove_not_tracked_is_ok(dokumenty_app_dir, monkeypatch, capsys):
    def _raise(*a, **k):
        raise tracking.DocumentNotTrackedError("nope")

    monkeypatch.setattr(_cli.tracking, "group_remove", _raise)
    _cli.cmd_group_remove(argparse.Namespace(codexis_id="ZAK1", name=["Skupina"]))
    assert "nic k odebrání" in capsys.readouterr().err


def test_related_remove_type_not_in_existing_is_ok(dokumenty_app_dir, capsys):
    # Document is tracked and has related-tracking enabled, but not for the
    # requested type - the "type not in existing" branch, not the
    # "document not tracked at all" branch.
    state.save_state(
        "ZAK1",
        {
            "codexisId": "ZAK1",
            "name": "Zákon ZAK1",
            "added_on": "2026-03-01T00:00:00Z",
            "changes": [],
            "related_tracking": {"enabled": True, "types": ["JUDIKATURA"]},
        },
    )
    _cli.cmd_related_remove(
        argparse.Namespace(codexis_id="ZAK1", type="PROVADECI_PREDPIS")
    )
    assert "nic k odebrání" in capsys.readouterr().err
