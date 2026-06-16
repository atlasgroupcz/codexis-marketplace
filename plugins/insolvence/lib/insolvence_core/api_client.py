"""Client for the public ISIR SOAP web service (no API key, no auth).

Uses the public "ISIR WS pro veřejnost / ČÚZK" service:

    https://isir.justice.cz:8443/isir_cuzk_ws/IsirWsCuzkService

operation ``getIsirWsCuzkData`` — per-subject lookup by IČO / RČ / name+DOB.
The response carries one ``<data>`` row per insolvency proceeding. Plain HTTP
POST of a hand-built SOAP envelope, parsed with the stdlib XML parser — no SOAP
library and no third-party dependencies.
"""

import os
import time
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET
from xml.sax.saxutils import escape

from .exceptions import ApiHttpError, ApiNetworkError, ApiParseError

WS_URL = (
    os.environ.get("CODEXIS_PLUGIN_INSOLVENCE_ISIR_WS_URL")
    or "https://isir.justice.cz:8443/isir_cuzk_ws/IsirWsCuzkService"
)
TYPES_NS = "http://isirws.cca.cz/types/"
TIMEOUT = 30
RETRIES = 2

# nazevOrganizace → standard court code used in spisová značka prefixes.
COURT_CODES = {
    "Městský soud v Praze": "MSPH",
    "Krajský soud v Praze": "KSPH",
    "Krajský soud v Českých Budějovicích": "KSCB",
    "Krajský soud v Plzni": "KSPL",
    "Krajský soud v Ústí nad Labem": "KSUL",
    "Krajský soud v Hradci Králové": "KSHK",
    "Krajský soud v Brně": "KSBR",
    "Krajský soud v Ostravě": "KSOS",
    "Vrchní soud v Praze": "VSPH",
    "Vrchní soud v Olomouci": "VSOL",
    "Nejvyšší soud": "NS",
    "Nejvyšší soud ČR": "NS",
}

# druhStavKonkursu → coarse bucket + human label. Open enums: unknown → "neurceno".
_DRUH_MAP = {
    "KONKURS": "konkurs",
    "ODDLUŽENÍ": "oddluzeni",
    "ODDLUZENI": "oddluzeni",
    "REORGANIZACE": "reorganizace",
    "MORATORIUM": "moratorium",
}
_STAV_LABELS = {
    "KONKURS": "Konkurs",
    "ODDLUŽENÍ": "Oddlužení",
    "ODDLUZENI": "Oddlužení",
    "REORGANIZACE": "Reorganizace",
    "MORATORIUM": "Moratorium",
    "ODSKRTNUTA": "Řízení ukončeno",
}

# getIsirWsCuzkDataRequest children — XSD enforces THIS sequence order.
_REQUEST_ORDER = [
    "ic",
    "rc",
    "druhVec",
    "bcVec",
    "rocnik",
    "nazevOsoby",
    "jmeno",
    "datumNarozeni",
    "maxPocetVysledku",
    "filtrAktualniRizeni",
    "vyhledatPresnouShoduJmen",
    "vyhledatBezDiakritiky",
    "maxRelevanceVysledku",
]


# ── XML helpers ──────────────────────────────────────────────────────────────


def _strip_ns(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def _child_text(el, name: str) -> str:
    for c in el:
        if _strip_ns(c.tag) == name:
            return (c.text or "").strip()
    return ""


def _find_all_local(root, name: str) -> list:
    return [el for el in root.iter() if _strip_ns(el.tag) == name]


def _strip_date(value: str) -> str:
    """ISIR dates arrive as 'YYYY-MM-DDZ' — drop the trailing Z."""
    value = (value or "").strip()
    return value[:-1] if value.endswith("Z") else value


# ── request / transport ────────────────────────────────────────────────────


def _envelope(fields: dict) -> str:
    """Build the SOAP envelope, emitting children in the canonical XSD order."""
    body = "".join(
        f"<{tag}>{escape(str(fields[tag]))}</{tag}>"
        for tag in _REQUEST_ORDER
        if fields.get(tag) not in (None, "")
    )
    return (
        '<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" '
        f'xmlns:typ="{TYPES_NS}">'
        "<soapenv:Body>"
        f"<typ:getIsirWsCuzkDataRequest>{body}</typ:getIsirWsCuzkDataRequest>"
        "</soapenv:Body></soapenv:Envelope>"
    )


def _post(envelope: str) -> bytes:
    req = urllib.request.Request(
        WS_URL,
        data=envelope.encode("utf-8"),
        headers={"Content-Type": "text/xml; charset=UTF-8", "SOAPAction": ""},
        method="POST",
    )
    last_err = None
    for attempt in range(RETRIES + 1):
        try:
            with urllib.request.urlopen(req, timeout=TIMEOUT) as resp:
                return resp.read()
        except urllib.error.HTTPError as e:
            if e.code >= 500 and attempt < RETRIES:
                last_err = e
                time.sleep(1 + attempt)
                continue
            body = e.read().decode("utf-8", "replace")[:500]
            raise ApiHttpError(e.code, f"ISIR WS vrátilo HTTP {e.code}: {body}")
        except urllib.error.URLError as e:
            if attempt < RETRIES:
                last_err = e
                time.sleep(1 + attempt)
                continue
            raise ApiNetworkError(f"Nepodařilo se kontaktovat ISIR WS: {e.reason}")
    raise ApiNetworkError(f"ISIR WS nedostupné: {last_err}")


def _query(fields: dict):
    """POST a getIsirWsCuzkData request; return (data_elements, relevance:int).

    Empty results (kodChyby WS2) return ([], relevance). Faults / other error
    codes raise ApiParseError.
    """
    try:
        root = ET.fromstring(_post(fields if isinstance(fields, str) else _envelope(fields)))
    except ET.ParseError as e:
        raise ApiParseError(f"Neplatná XML odpověď ISIR WS: {e}")

    faults = _find_all_local(root, "faultstring") or _find_all_local(root, "Fault")
    if faults:
        msg = (faults[0].text or "SOAP Fault").strip()
        raise ApiParseError(f"ISIR WS SOAP Fault: {msg}")

    stav_els = _find_all_local(root, "stav")
    relevance = 0
    if stav_els:
        stav = stav_els[0]
        kod = _child_text(stav, "kodChyby")
        rel = _child_text(stav, "relevanceVysledku")
        relevance = int(rel) if rel.isdigit() else 0
        if kod and kod != "WS2":
            text = _child_text(stav, "textChyby") or kod
            raise ApiParseError(f"ISIR WS chyba {kod}: {text}")
        if kod == "WS2":
            return [], relevance

    return _find_all_local(root, "data"), relevance


# ── row mapping ──────────────────────────────────────────────────────────────


def _row_to_rizeni(el) -> dict:
    organizace = _child_text(el, "nazevOrganizace")
    senat = _child_text(el, "cisloSenatu")
    druh_vec = _child_text(el, "druhVec") or "INS"
    bc = _child_text(el, "bcVec")
    rocnik = _child_text(el, "rocnik")
    code = COURT_CODES.get(organizace, "")
    prefix = " ".join(p for p in [code, senat, druh_vec] if p)
    spisova_znacka = f"{prefix} {bc}/{rocnik}".strip() if bc or rocnik else prefix

    stav_raw = _child_text(el, "druhStavKonkursu")
    stav_key = stav_raw.upper()
    datum_ukonceni = _strip_date(_child_text(el, "datumPmUkonceniUpadku"))

    return {
        "spisova_znacka": spisova_znacka,
        "soud": organizace,
        "druh": _DRUH_MAP.get(stav_key, "neurceno"),
        "druh_raw": stav_raw,
        "stav": _STAV_LABELS.get(stav_key, stav_raw or "Insolvenční řízení"),
        "datum_zahajeni": _strip_date(_child_text(el, "datumPmZahajeniUpadku")),
        "datum_ukonceni": datum_ukonceni,
        "aktivni": not datum_ukonceni and stav_key != "ODSKRTNUTA",
        "url_detail": _child_text(el, "urlDetailRizeni"),
        "udalosti": [],
    }


# ── public lookups ─────────────────────────────────────────────────────────


def lookup_company(ico: str) -> dict:
    """Look up insolvency proceedings for a company by IČO.

    Returns {"nazev": str, "rizeni": [<proceeding>...], "relevance": int}.
    """
    rows, relevance = _query({"ic": ico, "maxPocetVysledku": 200})
    nazev = _child_text(rows[0], "nazevOsoby") if rows else ""
    return {
        "nazev": nazev,
        "rizeni": [_row_to_rizeni(el) for el in rows],
        "relevance": relevance,
    }


def lookup_person(prijmeni: str, jmeno: str, datum_narozeni: str) -> dict:
    """Look up insolvency proceedings for a natural person by surname + given + DOB.

    Only trusts matches where the date of birth actually matched
    (relevanceVysledku 1/4/5); surname-only fallbacks (6/7) are treated as
    "no match for this person" to avoid false positives.
    """
    rows, relevance = _query(
        {
            "nazevOsoby": prijmeni,
            "jmeno": jmeno or None,
            "datumNarozeni": datum_narozeni,
            "maxPocetVysledku": 200,
            "vyhledatPresnouShoduJmen": "T",
            "vyhledatBezDiakritiky": "T",
        }
    )
    if relevance not in (1, 4, 5):
        return {"rizeni": [], "relevance": relevance}
    rizeni = [
        _row_to_rizeni(el)
        for el in rows
        if _strip_date(_child_text(el, "datumNarozeni")) == datum_narozeni
    ]
    return {"rizeni": rizeni, "relevance": relevance}
