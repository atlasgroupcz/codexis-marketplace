"""ISIR SOAP response parsing - deterministic, no network (canned XML)."""

import xml.etree.ElementTree as ET

import pytest
from insolvence_core import api_client
from insolvence_core.exceptions import ApiParseError

_COMPANY_XML = b"""<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
 <soap:Body>
  <ns2:getIsirWsCuzkDataResponse xmlns:ns2="http://isirws.cca.cz/types/">
   <data>
    <ic>04863313</ic>
    <cisloSenatu>78</cisloSenatu>
    <druhVec>INS</druhVec>
    <bcVec>21517</bcVec>
    <rocnik>2021</rocnik>
    <nazevOrganizace>M\xc4\x9bstsk\xc3\xbd soud v Praze</nazevOrganizace>
    <nazevOsoby>J.O. Investment s.r.o.</nazevOsoby>
    <druhStavKonkursu>KONKURS</druhStavKonkursu>
    <urlDetailRizeni>https://isir.justice.cz/isir/ueu/evidence_upadcu_detail.do?id=ABC</urlDetailRizeni>
    <datumPmZahajeniUpadku>2023-02-08Z</datumPmZahajeniUpadku>
   </data>
   <stav><pocetVysledku>1</pocetVysledku><relevanceVysledku>2</relevanceVysledku></stav>
  </ns2:getIsirWsCuzkDataResponse>
 </soap:Body>
</soap:Envelope>"""

_EMPTY_XML = b"""<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
 <soap:Body>
  <ns2:getIsirWsCuzkDataResponse xmlns:ns2="http://isirws.cca.cz/types/">
   <stav><pocetVysledku>0</pocetVysledku><kodChyby>WS2</kodChyby><textChyby>Prazdny vysledek</textChyby></stav>
  </ns2:getIsirWsCuzkDataResponse>
 </soap:Body>
</soap:Envelope>"""

_FAULT_XML = b"""<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
 <soap:Body>
  <soap:Fault><faultcode>soap:Client</faultcode><faultstring>Bad element order</faultstring></soap:Fault>
 </soap:Body>
</soap:Envelope>"""


def _person_xml(relevance, rows):
    data = "".join(
        f"<data><nazevOsoby>{r['p']}</nazevOsoby><jmeno>{r['j']}</jmeno>"
        f"<datumNarozeni>{r['d']}Z</datumNarozeni><cisloSenatu>20</cisloSenatu>"
        f"<druhVec>INS</druhVec><bcVec>{r['bc']}</bcVec><rocnik>2024</rocnik>"
        f"<nazevOrganizace>Krajský soud v Brně</nazevOrganizace>"
        f"<druhStavKonkursu>ODDLUZENI</druhStavKonkursu></data>"
        for r in rows
    )
    return (
        '<?xml version="1.0" encoding="UTF-8"?>'
        '<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"><soap:Body>'
        '<ns2:getIsirWsCuzkDataResponse xmlns:ns2="http://isirws.cca.cz/types/">'
        f"{data}<stav><pocetVysledku>{len(rows)}</pocetVysledku>"
        f"<relevanceVysledku>{relevance}</relevanceVysledku></stav>"
        "</ns2:getIsirWsCuzkDataResponse></soap:Body></soap:Envelope>"
    ).encode()


@pytest.fixture
def fake_post(monkeypatch):
    sent = []

    def _set(xml_bytes):
        def _post(envelope):
            sent.append(envelope)
            return xml_bytes

        monkeypatch.setattr(api_client, "_post", _post)
        return sent

    return _set


def _request_fields(envelope: str) -> dict:
    request = next(
        el
        for el in ET.fromstring(envelope).iter()
        if el.tag.endswith("getIsirWsCuzkDataRequest")
    )
    return {child.tag: child.text for child in request}


class TestCompanyParse:
    def test_maps_proceeding(self, fake_post):
        fake_post(_COMPANY_XML)
        result = api_client.lookup_company("04863313")
        assert set(result) == {"nazev", "rizeni"}
        assert result["nazev"] == "J.O. Investment s.r.o."
        assert len(result["rizeni"]) == 1
        r = result["rizeni"][0]
        assert r["spisova_znacka"] == "MSPH 78 INS 21517/2021"
        assert r["soud"] == "Městský soud v Praze"
        assert r["druh"] == "konkurs"
        assert r["stav"] == "Konkurs"
        assert r["datum_zahajeni"] == "2023-02-08"  # trailing Z stripped
        assert r["aktivni"] is True
        assert r["url_detail"].endswith("id=ABC")

    def test_empty_result(self, fake_post):
        fake_post(_EMPTY_XML)
        result = api_client.lookup_company("04863313")
        assert result["rizeni"] == []

    def test_service_error_code_raises(self, fake_post):
        fake_post(_EMPTY_XML.replace(b"WS2", b"WS1"))
        with pytest.raises(ApiParseError, match="WS1"):
            api_client.lookup_company("04863313")

    def test_soap_fault_raises(self, fake_post):
        fake_post(_FAULT_XML)
        with pytest.raises(ApiParseError):
            api_client.lookup_company("04863313")


class TestPersonParse:
    def test_request_is_capped_at_surname_given_name_and_birthdate(self, fake_post):
        sent = fake_post(_EMPTY_XML)
        api_client.lookup_person("Novák", "Jan", "1976-10-09")
        fields = _request_fields(sent[0])
        assert fields["nazevOsoby"] == "Novák"
        assert fields["jmeno"] == "Jan"
        assert fields["datumNarozeni"] == "1976-10-09"
        assert fields["maxRelevanceVysledku"] == "4"
        assert list(fields)[-1] == "maxRelevanceVysledku"  # XSD sequence order

    def test_request_without_given_name_is_capped_at_surname_and_birthdate(
        self, fake_post
    ):
        sent = fake_post(_EMPTY_XML)
        api_client.lookup_person("Novák", "", "1976-10-09")
        fields = _request_fields(sent[0])
        assert "jmeno" not in fields
        assert fields["maxRelevanceVysledku"] == "5"

    def test_no_exact_match_is_an_empty_list(self, fake_post):
        fake_post(_EMPTY_XML)
        result = api_client.lookup_person("Novák", "Jan", "1980-05-01")
        assert result == {"rizeni": []}

    def test_exact_match_maps_proceeding(self, fake_post):
        fake_post(
            _person_xml(
                4, [{"p": "Novák", "j": "Jan", "d": "1976-10-09", "bc": "12844"}]
            )
        )
        result = api_client.lookup_person("Novák", "Jan", "1976-10-09")
        assert set(result) == {"rizeni"}
        assert len(result["rizeni"]) == 1
        assert result["rizeni"][0]["spisova_znacka"] == "KSBR 20 INS 12844/2024"
        assert result["rizeni"][0]["druh"] == "oddluzeni"

    def test_co_debtor_row_is_dropped(self, fake_post):
        # A joint oddlužení returns the spouse's row with the debtor's proceeding.
        fake_post(
            _person_xml(
                4,
                [
                    {"p": "Novák", "j": "Jan", "d": "1976-10-09", "bc": "12844"},
                    {"p": "Nováková", "j": "Andrea", "d": "1980-07-23", "bc": "12844"},
                ],
            )
        )
        result = api_client.lookup_person("Novák", "Jan", "1976-10-09")
        assert [r["spisova_znacka"] for r in result["rizeni"]] == [
            "KSBR 20 INS 12844/2024"
        ]
