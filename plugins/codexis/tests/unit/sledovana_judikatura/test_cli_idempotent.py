"""CLI-layer idempotency: dup area-add, out-of-range area-remove, missing delete → OK."""

import argparse
import json
import os

from sledovana_judikatura_core import tracking

from tests.harness.script_loader import load_script


def _load_cli():
    path = os.path.join(
        os.path.dirname(__file__), "..", "..", "..", "bin", "cdx-sledovana-judikatura"
    )
    return load_script("cdx_sledovana_judikatura", path)


_cli = _load_cli()


def test_area_add_duplicate_is_ok(judikatura_app_dir, capsys):
    u, _ = tracking.init("T")
    tracking.area_add(u, "Náhrada škody")
    _cli.cmd_area_add(argparse.Namespace(uuid=u, name=["Náhrada", "škody"]))
    assert "už existuje" in capsys.readouterr().err


def test_area_remove_out_of_range_is_ok(judikatura_app_dir, capsys):
    u, _ = tracking.init("T")
    _cli.cmd_area_remove(argparse.Namespace(uuid=u, index=5))
    assert "nic k odebrání" in capsys.readouterr().err


def test_delete_missing_is_ok(judikatura_app_dir, capsys):
    _cli.cmd_delete(argparse.Namespace(uuid="nonexistent"))
    assert "neexistuje" in capsys.readouterr().err


def test_init_existing_hints_on_notes(judikatura_app_dir, capsys):
    tracking.init("Téma")
    _cli.cmd_init(argparse.Namespace(name="Téma", notes="nový kontext"))
    err = capsys.readouterr().err
    assert "už existuje" in err
    assert "note add" in err
    # notes not mutated on re-init
    _uuid, data = tracking.find_topic_by_name("Téma")
    assert data["notes"] == []


def test_save_report_id_on_stdout_ok_on_stderr(judikatura_app_dir, tmp_path, capsys):
    u, _ = tracking.init("T")
    report_path = tmp_path / "report.json"
    report_path.write_text(json.dumps({"report_id": "2026-04-20", "found_count": 3}))
    _cli.cmd_save_report(argparse.Namespace(uuid=u, file=str(report_path)))
    out, err = capsys.readouterr()
    assert out.strip() == "2026-04-20"
    assert "OK: report '2026-04-20' uložen" in err
