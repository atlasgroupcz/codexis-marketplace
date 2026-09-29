"""`lustrace` prints only what the ISIR lookup found - no match metadata."""

import argparse
import json
import os

from insolvence_core import api_client

from tests.harness.script_loader import load_script

_EMPTY_XML = (
    b'<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"><soap:Body>'
    b'<ns2:getIsirWsCuzkDataResponse xmlns:ns2="http://isirws.cca.cz/types/">'
    b"<stav><kodChyby>WS2</kodChyby><textChyby>Prazdny vysledek</textChyby></stav>"
    b"</ns2:getIsirWsCuzkDataResponse></soap:Body></soap:Envelope>"
)

_cli = load_script(
    "insolvence_cli",
    os.path.join(os.path.dirname(__file__), "..", "..", "bin", "insolvence-cli"),
)


def test_lustrace_osoba_without_match_prints_empty_proceedings(monkeypatch, capsys):
    monkeypatch.setattr(api_client, "_post", lambda envelope: _EMPTY_XML)
    _cli.cmd_lustrace_osoba(
        argparse.Namespace(prijmeni="Novák", jmeno="Jan", datum_narozeni="1.5.1980")
    )
    assert json.loads(capsys.readouterr().out) == {"rizeni": []}
