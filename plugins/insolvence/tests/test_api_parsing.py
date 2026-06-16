"""ISIR SOAP response parsing — deterministic, no network (canned XML)."""

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
    ).encode("utf-8")


@pytest.fixture
def fake_post(monkeypatch):
    def _set(xml_bytes):
        monkeypatch.setattr(api_client, "_post", lambda envelope: xml_bytes)
    return _set


class TestCompanyParse:
    def test_maps_proceeding(self, fake_post):
        fake_post(_COMPANY_XML)
        result = api_client.lookup_company("04863313")
        assert result["nazev"] == "J.O. Investment s.r.o."
        assert result["relevance"] == 2
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

    def test_soap_fault_raises(self, fake_post):
        fake_post(_FAULT_XML)
        with pytest.raises(ApiParseError):
            api_client.lookup_company("04863313")


class TestPersonParse:
    def test_weak_relevance_discarded(self, fake_post):
        # relevance 7 = surname-only fallback → no trustworthy match
        fake_post(_person_xml(7, [{"p": "Novak", "j": "Petr", "d": "1990-01-01", "bc": "1"}]))
        result = api_client.lookup_person("Novak", "Jan", "1980-05-01")
        assert result["rizeni"] == []
        assert result["relevance"] == 7

    def test_dob_match_filters_rows(self, fake_post):
        # relevance 5 (surname+DOB); only the row with the requested DOB is kept
        fake_post(
            _person_xml(
                5,
                [
                    {"p": "Novak", "j": "Jan", "d": "1980-05-01", "bc": "10"},
                    {"p": "Novak", "j": "Jan", "d": "1975-03-03", "bc": "20"},
                ],
            )
        )
        result = api_client.lookup_person("Novak", "Jan", "1980-05-01")
        assert len(result["rizeni"]) == 1
        assert result["rizeni"][0]["spisova_znacka"] == "KSBR 20 INS 10/2024"
        assert result["rizeni"][0]["druh"] == "oddluzeni"
