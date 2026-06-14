from __future__ import annotations

import io
import json
import sys
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest.mock import Mock, patch
from urllib import error


ROOT = Path(__file__).resolve().parents[1]
LIB = ROOT / "plugins" / "ares" / "lib"
sys.path.insert(0, str(LIB))

from ares_cli import cli, formatters
from ares_cli.client import AresClient
from ares_cli.errors import AresCliError
from ares_cli.service import AresService


class FakeClient:
    def __init__(self) -> None:
        self.get_calls: list[str] = []
        self.post_calls: list[tuple[str, dict]] = []
        self.get_responses: dict[str, dict] = {}
        self.post_responses: dict[str, dict] = {}

    def get_json(self, path: str) -> dict:
        self.get_calls.append(path)
        if path not in self.get_responses:
            raise AssertionError(f"Unexpected GET {path}")
        return self.get_responses[path]

    def post_json(self, path: str, payload: dict) -> dict:
        self.post_calls.append((path, payload))
        if path not in self.post_responses:
            raise AssertionError(f"Unexpected POST {path}")
        return self.post_responses[path]


class AresServiceTests(unittest.TestCase):
    def test_search_posts_name_filter_and_echoes_request(self) -> None:
        client = FakeClient()
        client.post_responses["/ekonomicke-subjekty/vyhledat"] = {
            "pocetCelkem": 1,
            "ekonomickeSubjekty": [
                {
                    "obchodniJmeno": "ATLAS spol. s r.o.",
                    "ico": "00528609",
                    "sidlo": {"textovaAdresa": "K Pustinám 64, Pardubice"},
                    "pravniForma": "112",
                    "primarniZdroj": "ros",
                    "seznamRegistraci": {"stavZdrojeVr": "AKTIVNI"},
                }
            ],
        }

        result = AresService(client).search("  ATLAS  ", limit=1)

        self.assertEqual(
            client.post_calls,
            [
                (
                    "/ekonomicke-subjekty/vyhledat",
                    {"obchodniJmeno": "ATLAS", "pocet": 50, "start": 0},
                )
            ],
        )
        self.assertEqual(result["echo"]["query"], "ATLAS")
        self.assertEqual(result["echo"]["limit"], 1)
        self.assertEqual(result["echo"]["fetchLimit"], 50)
        self.assertEqual(result["echo"]["ranking"], "local_relevance")
        self.assertEqual(result["kandidati"][0]["nazev"], "ATLAS spol. s r.o.")
        self.assertEqual(result["kandidati"][0]["stavRegistraci"]["stavZdrojeVr"], "AKTIVNI")

    def test_search_reranks_exact_legal_name_before_subsidiaries(self) -> None:
        client = FakeClient()
        client.post_responses["/ekonomicke-subjekty/vyhledat"] = {
            "pocetCelkem": 4,
            "ekonomickeSubjekty": [
                {"obchodniJmeno": "ČEZ Recyklace, s.r.o.", "ico": "03479919", "pravniForma": "112"},
                {"obchodniJmeno": "ČEZ ESCO, a.s.", "ico": "03592880", "pravniForma": "121"},
                {"obchodniJmeno": "Nadace ČEZ", "ico": "26721511", "pravniForma": "117"},
                {
                    "obchodniJmeno": "ČEZ, a. s.",
                    "ico": "45274649",
                    "pravniForma": "121",
                    "seznamRegistraci": {"stavZdrojeVr": "AKTIVNI", "stavZdrojeRzp": "AKTIVNI"},
                },
            ],
        }

        result = AresService(client).search("ČEZ", limit=2)

        self.assertEqual(result["kandidati"][0]["ico"], "45274649")
        self.assertEqual(len(result["kandidati"]), 2)
        self.assertEqual(result["pocetCelkem"], 4)
        self.assertEqual(result["pocetVraceno"], 2)

    def test_search_respects_large_limit_as_fetch_limit(self) -> None:
        client = FakeClient()
        client.post_responses["/ekonomicke-subjekty/vyhledat"] = {"ekonomickeSubjekty": []}

        AresService(client).search("ATLAS", limit=75)

        self.assertEqual(
            client.post_calls,
            [("/ekonomicke-subjekty/vyhledat", {"obchodniJmeno": "ATLAS", "pocet": 75, "start": 0})],
        )

    def test_search_rejects_empty_query(self) -> None:
        with self.assertRaisesRegex(AresCliError, "dotaz nesmí být prázdný"):
            AresService(FakeClient()).search("   ", limit=10)

    def test_search_rejects_limit_outside_api_range(self) -> None:
        service = AresService(FakeClient())
        for limit in (0, 1001):
            with self.subTest(limit=limit):
                with self.assertRaisesRegex(AresCliError, "--limit musí být v rozsahu 1 až 1000"):
                    service.search("ATLAS", limit=limit)

    def test_company_normalizes_short_ico_and_maps_company_card(self) -> None:
        client = FakeClient()
        client.get_responses["/ekonomicke-subjekty/00000001"] = {
            "ico": "00000001",
            "obchodniJmeno": "Test s.r.o.",
            "dic": "CZ00000001",
            "sidlo": {"textovaAdresa": "Testovací 1, Praha"},
            "pravniForma": "112",
            "datumVzniku": "2020-01-01",
            "datumAktualizace": "2026-06-01",
            "primarniZdroj": "ros",
            "seznamRegistraci": {"stavZdrojeRzp": "AKTIVNI"},
            "czNace": ["62010"],
        }

        result = AresService(client).company("1")

        self.assertEqual(client.get_calls, ["/ekonomicke-subjekty/00000001"])
        self.assertEqual(result["echo"]["ico"], "00000001")
        self.assertEqual(result["kartaSubjektu"]["nazev"], "Test s.r.o.")
        self.assertEqual(result["kartaSubjektu"]["sidlo"], "Testovací 1, Praha")
        self.assertEqual(result["kartaSubjektu"]["registrace"]["stavZdrojeRzp"], "AKTIVNI")

    def test_ico_validation_rejects_missing_or_too_long_digits(self) -> None:
        service = AresService(FakeClient())
        for ico in ("abc", "", "123456789"):
            with self.subTest(ico=ico):
                with self.assertRaisesRegex(AresCliError, "IČO musí obsahovat 1 až 8 číslic"):
                    service.company(ico)

    def test_raw_returns_original_json_without_echo(self) -> None:
        client = FakeClient()
        raw = {"ico": "27082440", "obchodniJmeno": "Alza.cz a.s."}
        client.get_responses["/ekonomicke-subjekty/27082440"] = raw

        result = AresService(client).raw("27082440", source="basic")

        self.assertIs(result, raw)
        self.assertNotIn("echo", result)
        self.assertEqual(client.get_calls, ["/ekonomicke-subjekty/27082440"])

    def test_officers_maps_vr_nested_fields(self) -> None:
        client = FakeClient()
        client.get_responses["/ekonomicke-subjekty-vr/27082440"] = {
            "icoId": "27082440",
            "zaznamy": [
                {
                    "obchodniJmeno": [{"hodnota": "Alza.cz a.s."}],
                    "ico": [{"hodnota": "27082440"}],
                    "spisovaZnacka": [{"soud": "MSPH", "oddil": "B", "vlozka": 8573}],
                    "zakladniKapital": [{"vklad": {"hodnota": "2000000;00", "typObnos": "KORUNY"}}],
                    "statutarniOrgany": [
                        {
                            "nazevOrganu": "Statutární orgán",
                            "clenoveOrganu": [
                                {
                                    "fyzickaOsoba": {"jmeno": "ALEŠ", "prijmeni": "ZAVORAL"},
                                    "clenstvi": {
                                        "funkce": {"nazev": "předseda", "vznikFunkce": "2022-11-16"},
                                        "clenstvi": {"vznikClenstvi": "2022-11-09"},
                                    },
                                }
                            ],
                            "zpusobJednani": [{"hodnota": "Každý člen jedná samostatně."}],
                        }
                    ],
                    "exekuce": [{"hodnota": "Bez záznamu test"}],
                    "insolvence": [{"datumZapisu": "2024-01-01"}],
                    "konkursy": [{"typKonkursu": "TEST"}],
                }
            ],
        }

        result = AresService(client).officers("27082440")
        record = result["zaznamy"][0]

        self.assertEqual(record["spisovaZnacka"][0]["znacka"], "B 8573 MSPH")
        self.assertEqual(record["zakladniKapital"][0]["vklad"], "2000000;00 KORUNY")
        self.assertEqual(record["statutarniOrgany"][0]["clenove"][0]["funkce"], "předseda")
        self.assertEqual(record["zpusobJednani"], ["Každý člen jedná samostatně."])
        self.assertIn("insolvence", record)
        self.assertIn("konkursy", record)

    def test_trades_maps_rzp_nested_fields(self) -> None:
        client = FakeClient()
        client.get_responses["/ekonomicke-subjekty-rzp/27082440"] = {
            "icoId": "27082440",
            "zaznamy": [
                {
                    "obchodniJmeno": "Alza.cz a.s.",
                    "ico": "27082440",
                    "zivnostiStav": {"pocetAktivnich": 1, "pocetCelkem": 1},
                    "provozovnyStav": {"pocetAktivnich": 1, "pocetCelkem": 1},
                    "zivnosti": [
                        {
                            "predmetPodnikani": "Velkoobchod a maloobchod",
                            "druhZivnosti": "L",
                            "datumVzniku": "2003-08-26",
                            "oboryCinnosti": [{"oborNazev": "Velkoobchod a maloobchod"}],
                            "odpovedniZastupci": [
                                {"jmeno": "Jan", "prijmeni": "Novák", "typAngazma": "ODPOVEDNY_ZASTUPCE_RZP"}
                            ],
                            "provozovny": [
                                {
                                    "icp": 101,
                                    "sidloProvozovny": {"textovaAdresa": "Jankovcova 53, Praha"},
                                    "platnostOd": "2020-01-01",
                                }
                            ],
                        }
                    ],
                }
            ],
        }

        result = AresService(client).trades("27082440")
        record = result["zaznamy"][0]
        trade = record["zivnosti"][0]

        self.assertEqual(record["zivnostiStav"]["pocetAktivnich"], 1)
        self.assertEqual(trade["predmetPodnikani"], "Velkoobchod a maloobchod")
        self.assertEqual(trade["obory"][0]["nazev"], "Velkoobchod a maloobchod")
        self.assertEqual(trade["odpovedniZastupci"][0]["jmeno"], "Jan Novák")
        self.assertEqual(trade["provozovny"][0]["sidlo"], "Jankovcova 53, Praha")


class FormatterEdgeCaseTests(unittest.TestCase):
    def test_formatters_omit_missing_empty_values(self) -> None:
        result = formatters.company_summary(
            {"ico": "12345678", "obchodniJmeno": "Minimal s.r.o.", "sidlo": {}},
            echo={"command": "ares company", "ico": "12345678"},
        )

        card = result["kartaSubjektu"]
        self.assertEqual(card["ico"], "12345678")
        self.assertNotIn("sidlo", card)
        self.assertNotIn("registrace", card)

    def test_address_fallback_builds_text_from_parts(self) -> None:
        result = formatters.search_summary(
            {
                "ekonomickeSubjekty": [
                    {
                        "obchodniJmeno": "Fallback s.r.o.",
                        "sidlo": {
                            "nazevUlice": "Testovací",
                            "cisloDomovni": 10,
                            "nazevObce": "Praha",
                            "psc": 11000,
                        },
                    }
                ]
            },
            echo={"command": "ares search"},
        )

        self.assertEqual(result["kandidati"][0]["sidlo"], "Testovací 10 Praha 11000")


class ClientErrorTests(unittest.TestCase):
    def test_http_error_body_is_communicated(self) -> None:
        body = json.dumps(
            {"kod": "CHYBA_VSTUPU", "subKod": "ICO", "popis": "Neplatné IČO"},
            ensure_ascii=False,
        ).encode("utf-8")
        http_error = error.HTTPError(
            "https://ares.example/ekonomicke-subjekty/99999999",
            400,
            "Bad Request",
            hdrs={},
            fp=io.BytesIO(body),
        )

        with patch("ares_cli.client.request.urlopen", side_effect=http_error):
            with self.assertRaises(AresCliError) as caught:
                AresClient(base_url="https://ares.example").get_json("/ekonomicke-subjekty/99999999")

        message = str(caught.exception)
        self.assertIn("GET /ekonomicke-subjekty/99999999", message)
        self.assertIn("HTTP 400", message)
        self.assertIn("CHYBA_VSTUPU", message)
        self.assertIn("ICO", message)
        self.assertIn("Neplatné IČO", message)

    def test_http_error_list_body_is_communicated(self) -> None:
        body = json.dumps(
            {
                "chyby": [
                    {"kod": "CHYBA_VSTUPU", "popis": "První chyba"},
                    {"kod": "NENALEZENO", "popis": "Druhá chyba"},
                ]
            },
            ensure_ascii=False,
        ).encode("utf-8")
        http_error = error.HTTPError("https://ares.example/x", 404, "Not Found", hdrs={}, fp=io.BytesIO(body))

        with patch("ares_cli.client.request.urlopen", side_effect=http_error):
            with self.assertRaises(AresCliError) as caught:
                AresClient(base_url="https://ares.example").get_json("/x")

        message = str(caught.exception)
        self.assertIn("HTTP 404", message)
        self.assertIn("CHYBA_VSTUPU - První chyba", message)
        self.assertIn("NENALEZENO - Druhá chyba", message)

    def test_invalid_json_response_is_reported(self) -> None:
        response = Mock()
        response.__enter__ = Mock(return_value=response)
        response.__exit__ = Mock(return_value=None)
        response.read.return_value = b"not json"

        with patch("ares_cli.client.request.urlopen", return_value=response):
            with self.assertRaisesRegex(AresCliError, "response is not valid JSON"):
                AresClient(base_url="https://ares.example").get_json("/x")


class CliMainTests(unittest.TestCase):
    def test_cli_outputs_json_from_service(self) -> None:
        fake_service = Mock()
        fake_service.company.return_value = {
            "echo": {"command": "ares company", "ico": "27082440"},
            "kartaSubjektu": {"nazev": "Alza.cz a.s."},
        }

        stdout = io.StringIO()
        with patch("ares_cli.cli.AresService", return_value=fake_service):
            with redirect_stdout(stdout):
                exit_code = cli.main(["company", "27082440"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(json.loads(stdout.getvalue())["kartaSubjektu"]["nazev"], "Alza.cz a.s.")
        fake_service.company.assert_called_once_with("27082440")

    def test_cli_prints_user_facing_errors_to_stderr(self) -> None:
        fake_service = Mock()
        fake_service.company.side_effect = AresCliError("GET /x: HTTP 404: NENALEZENO - Nenalezeno")

        stderr = io.StringIO()
        with patch("ares_cli.cli.AresService", return_value=fake_service):
            with redirect_stderr(stderr):
                exit_code = cli.main(["company", "27082440"])

        self.assertEqual(exit_code, 2)
        self.assertIn("ERROR: GET /x: HTTP 404: NENALEZENO - Nenalezeno", stderr.getvalue())

    def test_cli_rejects_removed_rpsh_source(self) -> None:
        stderr = io.StringIO()
        with redirect_stderr(stderr):
            with self.assertRaises(SystemExit) as caught:
                cli.main(["raw", "27082440", "--source", "rpsh"])

        self.assertEqual(caught.exception.code, 2)
        self.assertIn("invalid choice: 'rpsh'", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
