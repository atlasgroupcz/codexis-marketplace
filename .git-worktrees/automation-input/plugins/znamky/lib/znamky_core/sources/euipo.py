"""EUIPO official Trademark Search API adapter (OAuth2, user credentials).

Enabled once the user stores EUIPO credentials (settings.is_configured()). Gets
a bearer token via client-credentials, queries the official trademark-search API
and normalises hits into canonical candidates. Endpoint/field mapping follow the
documented EUIPO API and are overridable via env; parsing is tolerant so minor
schema differences don't break the watcher (confirm against live API + a key).
"""

import os
import urllib.parse

from . import base
from .. import settings

SOURCE = "EUIPO"
API_URL = (
    os.environ.get("CODEXIS_PLUGIN_ZNAMKY_EUIPO_API_URL")
    or "https://api.euipo.europa.eu/trademark-search"
).rstrip("/")
DETAIL_BASE = "https://www.tmdn.org/tmview/welcome#/tmview/detail/EM50"


def enabled() -> bool:
    return settings.is_configured()


def _first(rec: dict, *names):
    for n in names:
        if n in rec and rec[n] not in (None, ""):
            return rec[n]
    return ""


def _verbal(rec: dict) -> str:
    spec = rec.get("wordMarkSpecification")
    if isinstance(spec, dict) and spec.get("verbalElement"):
        return spec["verbalElement"]
    return str(_first(rec, "markVerbalElementText", "verbalElement", "tmName", "name"))


def _nice(rec: dict) -> list:
    val = _first(rec, "niceClasses", "niceClass", "classDescriptionDetails")
    if isinstance(val, list):
        return val
    return base.normalize_classes(str(val).replace(";", ",").split(",")) if val else []


def _applicant(rec: dict) -> str:
    apps = rec.get("applicants")
    if isinstance(apps, list) and apps:
        a = apps[0]
        return a.get("name", "") if isinstance(a, dict) else str(a)
    return str(_first(rec, "applicantName", "holderName"))


def parse_results(payload: dict) -> list:
    records = (
        payload.get("trademarks")
        or payload.get("tradeMarks")
        or payload.get("results")
        or []
    )
    out = []
    for rec in records:
        if not isinstance(rec, dict):
            continue
        number = str(_first(rec, "applicationNumber", "applicationNumberFormatted", "ST13"))
        out.append(
            base.make_candidate(
                source=SOURCE,
                source_id=number,
                mark_text=_verbal(rec),
                mark_kind=_first(rec, "markFeature", "markKind", "feature"),
                applicant=_applicant(rec),
                status=_first(rec, "status", "markCurrentStatusCode"),
                filing_date=_first(rec, "applicationDate", "filingDate"),
                nice_classes=_nice(rec),
                vienna_codes=_first(rec, "viennaClasses", "viennaClass") or [],
                office="EM",
                url_detail=f"{DETAIL_BASE}{number}" if number else "",
                image_url=_first(rec, "markImageURI", "logo", "imageUri"),
            )
        )
    return out


def search(query_text: str, nice_classes=None, limit: int = 50) -> list:
    """Query the EUIPO trademark-search API for marks whose verbal element matches."""
    client_id, client_secret = settings.read_credentials()
    token = settings.fetch_token(client_id, client_secret)
    rsql = f'wordMarkSpecification.verbalElement=="*{(query_text or "").strip()}*"'
    qs = urllib.parse.urlencode({"query": rsql, "size": max(1, min(limit, 100))})
    response = base.get_json(
        f"{API_URL}/trademarks?{qs}",
        SOURCE,
        headers={"Authorization": f"Bearer {token}", "X-IBM-Client-Id": client_id},
    )
    return parse_results(response)
