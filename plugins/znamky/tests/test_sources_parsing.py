"""Source adapters: canonical normalisation, TMview response parsing, dispatch."""

from znamky_core import sources
from znamky_core.sources import base, tmview


class TestBaseNormalisation:
    def test_office_to_territory(self):
        assert base.office_to_territory("CZ") == "CZ"
        assert base.office_to_territory("EM") == "EU"
        assert base.office_to_territory("WO") == "INT"

    def test_normalize_kind(self):
        assert base.normalize_kind("Word") == "word"
        assert base.normalize_kind("Figurative") == "figurative"
        assert base.normalize_kind("Stylized characters") == "combined"

    def test_normalize_classes(self):
        assert base.normalize_classes(["9", "42", "9", "x"]) == [9, 42]

    def test_dedupe(self):
        a = base.make_candidate(source="TMVIEW", source_id="1")
        b = base.make_candidate(source="TMVIEW", source_id="1")
        c = base.make_candidate(source="TMVIEW", source_id="2")
        assert len(base.dedupe([a, b, c])) == 2


class TestTmviewParse:
    PAYLOAD = {
        "fullResultCount": 1,
        "tradeMarks": [
            {
                "ST13": "CZ500000001",
                "tmName": "CODEXIS",
                "applicantName": "ATLAS GROUP",
                "tradeMarkOffice": "CZ",
                "applicationDate": "2019-03-01",
                "applicationNumber": "550123",
                "markFeature": "Word",
                "niceClass": "9,42",
                "viennaClass": ["26.4.1"],
                "status": "Registered",
                "markImageURI": "https://img/x.png",
            }
        ],
    }

    def test_maps_fields(self):
        cands = tmview.parse_results(self.PAYLOAD)
        assert len(cands) == 1
        c = cands[0]
        assert c["source"] == "TMVIEW"
        assert c["source_id"] == "CZ500000001"
        assert c["mark_text"] == "CODEXIS"
        assert c["mark_kind"] == "word"
        assert c["nice_classes"] == [9, 42]
        assert c["vienna_codes"] == ["26.4.1"]
        assert c["office"] == "CZ"
        assert c["territory"] == "CZ"
        assert c["image_url"] == "https://img/x.png"
        assert c["url_detail"].endswith("CZ500000001")

    def test_handles_alias_fields(self):
        cands = tmview.parse_results({"results": [{"stableId": "EM1", "name": "ACME", "officeCode": "EM"}]})
        assert cands[0]["source_id"] == "EM1"
        assert cands[0]["mark_text"] == "ACME"
        assert cands[0]["territory"] == "EU"

    def test_empty(self):
        assert tmview.parse_results({}) == []


class TestDispatch:
    def test_territory_offices(self):
        assert sources.territory_offices(["CZ"]) == ["CZ", "WO"]
        assert sources.territory_offices(["EU"]) == ["EM", "WO"]
        assert set(sources.territory_offices(["CZ", "EU"])) == {"CZ", "EM", "WO"}
        assert sources.territory_offices([]) == ["CZ", "EM", "WO"]
