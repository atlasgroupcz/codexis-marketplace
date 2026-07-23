"""CLI-layer idempotency: add-already-tracked and remove-not-tracked exit OK."""

import argparse
import importlib.machinery
import importlib.util
import os

from insolvence_core import tracking


def _load_cli():
    path = os.path.join(os.path.dirname(__file__), "..", "..", "bin", "insolvence-cli")
    loader = importlib.machinery.SourceFileLoader("insolvence_cli", path)
    spec = importlib.util.spec_from_file_location("insolvence_cli", path, loader=loader)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_cli = _load_cli()


def _raise_already(*a, **k):
    raise tracking.SubjectAlreadyTrackedError("dup")


def test_add_firma_already_tracked_is_ok(app_dir, monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking, "add_company", _raise_already)
    _cli.cmd_add_firma(argparse.Namespace(ico="00000000", label=""))
    assert "je již sledována" in capsys.readouterr().err


def test_add_firma_already_tracked_keeps_label_and_hints(app_dir, monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking.api_client, "lookup_company",
                        lambda ico: {"nazev": "Firma X", "rizeni": []})
    ico = "00000001"  # passes normalize_ico's mod-11 checksum
    tracking.add_company(ico, label="Původní")
    _cli.cmd_add_firma(argparse.Namespace(ico=ico, label="Nový"))
    err = capsys.readouterr().err
    assert tracking.find_company(ico)["label"] == "Původní"   # unchanged
    assert "set-label" in err


def test_add_osoba_already_tracked_is_ok(app_dir, monkeypatch, capsys):
    def _raise(*a, **k):
        raise tracking.SubjectAlreadyTrackedError("dup")
    monkeypatch.setattr(_cli.tracking, "add_person", _raise)
    _cli.cmd_add_osoba(argparse.Namespace(
        prijmeni="Novák", jmeno="Jan", datum_narozeni="1980-01-01", label=""))
    assert "je již sledována" in capsys.readouterr().err


def test_add_osoba_already_tracked_keeps_label_and_hints(app_dir, monkeypatch, capsys):
    monkeypatch.setattr(_cli.tracking.api_client, "lookup_person",
                        lambda prijmeni, jmeno, datum_narozeni: {"rizeni": []})
    tracking.add_person("Novák", "Jan", "1980-01-01", label="Původní")
    _cli.cmd_add_osoba(argparse.Namespace(
        prijmeni="Novák", jmeno="Jan", datum_narozeni="1980-01-01", label="Nový"))
    err = capsys.readouterr().err
    assert tracking.find_person("Novák", "Jan", "1980-01-01")["label"] == "Původní"
    assert "set-label" in err


def test_remove_not_tracked_is_ok(app_dir, capsys):
    _cli.cmd_remove(argparse.Namespace(id="00000000"))
    assert "nic k odebrání" in capsys.readouterr().err
