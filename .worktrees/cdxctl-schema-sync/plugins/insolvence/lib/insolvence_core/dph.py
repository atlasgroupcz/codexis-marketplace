"""Client for the MF/GFŘ public VAT-payer reliability registry (no key, no auth).

SOAP service "Registr plátců DPH – rozhraní pro veřejnost":

    https://adisrws.mfcr.cz/adistc/axis2/services/rozhraniCRPDPH.rozhraniCRPDPHSOAP

operation ``getStatusNespolehlivyPlatce`` — looks up a subject by DIČ (the bare
IČO works) and returns the unreliable-payer flag (+ since when), the competent
tax office and the published bank accounts. Plain stdlib HTTP + XML, no deps.
"""

import os
import time
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET
from xml.sax.saxutils import escape

from .exceptions import ApiHttpError, ApiNetworkError, ApiParseError

WS_URL = (
    os.environ.get("CODEXIS_PLUGIN_INSOLVENCE_DPH_WS_URL")
    or "https://adisrws.mfcr.cz/adistc/axis2/services/rozhraniCRPDPH.rozhraniCRPDPHSOAP"
)
NS = "http://adis.mfcr.cz/rozhraniCRPDPH/"
TIMEOUT = 30
RETRIES = 2


def _strip_ns(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def _find(root, name: str):
    for el in root.iter():
        if _strip_ns(el.tag) == name:
            return el
    return None


def _find_all(root, name: str) -> list:
    return [el for el in root.iter() if _strip_ns(el.tag) == name]


def _envelope(dic: str) -> str:
    return (
        '<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" '
        f'xmlns:roz="{NS}">'
        "<soapenv:Body>"
        f"<roz:StatusNespolehlivyPlatceRequest><roz:dic>{escape(dic)}</roz:dic>"
        "</roz:StatusNespolehlivyPlatceRequest>"
        "</soapenv:Body></soapenv:Envelope>"
    )


def _post(dic: str) -> bytes:
    req = urllib.request.Request(
        WS_URL,
        data=_envelope(dic).encode("utf-8"),
        headers={
            "Content-Type": "text/xml; charset=UTF-8",
            "SOAPAction": "getStatusNespolehlivyPlatce",
        },
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
            raise ApiHttpError(e.code, f"Registr DPH vrátil HTTP {e.code}")
        except urllib.error.URLError as e:
            if attempt < RETRIES:
                last_err = e
                time.sleep(1 + attempt)
                continue
            raise ApiNetworkError(f"Nepodařilo se kontaktovat registr DPH: {e.reason}")
    raise ApiNetworkError(f"Registr DPH nedostupný: {last_err}")


def _format_account(ucet_el) -> dict:
    datum = ucet_el.get("datumZverejneni", "")
    std = _find(ucet_el, "standardniUcet")
    if std is not None:
        predcisli = (std.get("predcisli") or "").strip()
        cislo = (std.get("cislo") or "").strip()
        kod = (std.get("kodBanky") or "").strip()
        ucet = f"{predcisli}-{cislo}/{kod}" if predcisli else f"{cislo}/{kod}"
        return {"ucet": ucet, "iban": False, "datum_zverejneni": datum}
    nestd = _find(ucet_el, "nestandardniUcet")
    if nestd is not None:
        return {"ucet": (nestd.get("cislo") or "").strip(), "iban": True, "datum_zverejneni": datum}
    return {"ucet": "", "iban": False, "datum_zverejneni": datum}


def lookup_dph(ico: str) -> dict:
    """Look up the VAT-payer reliability status for a subject by IČO/DIČ.

    Returns:
        {
          "dic": str, "found": bool, "je_platce": bool,
          "nespolehlivy": bool | None, "nespolehlivy_od": str,
          "cislo_fu": str, "ucty": [ {ucet, iban, datum_zverejneni} ],
          "generovano": str,
        }
    """
    dic = "".join(ch for ch in (ico or "") if ch.isdigit())
    if not dic:
        raise ApiParseError("Neplatné IČO/DIČ pro kontrolu plátce DPH.")

    try:
        root = ET.fromstring(_post(dic))
    except ET.ParseError as e:
        raise ApiParseError(f"Neplatná XML odpověď registru DPH: {e}")

    status = _find(root, "status")
    if status is not None and status.get("statusCode") not in (None, "0"):
        raise ApiParseError(
            f"Registr DPH chyba {status.get('statusCode')}: {status.get('statusText') or ''}"
        )

    generovano = status.get("odpovedGenerovana", "") if status is not None else ""
    sp = _find(root, "statusPlatceDPH")
    if sp is None:
        return {
            "dic": dic, "found": False, "je_platce": False,
            "nespolehlivy": None, "nespolehlivy_od": "", "cislo_fu": "",
            "ucty": [], "generovano": generovano,
        }

    flag = (sp.get("nespolehlivyPlatce") or "").upper()
    je_platce = flag in ("ANO", "NE")
    nespolehlivy = True if flag == "ANO" else (False if flag == "NE" else None)
    ucty = [_format_account(u) for u in _find_all(sp, "ucet")]

    return {
        "dic": sp.get("dic") or dic,
        "found": True,
        "je_platce": je_platce,
        "nespolehlivy": nespolehlivy,
        "nespolehlivy_od": sp.get("nespolehlivyOd") or "",
        "cislo_fu": sp.get("cisloFu") or "",
        "ucty": [u for u in ucty if u["ucet"]],
        "generovano": generovano,
    }
