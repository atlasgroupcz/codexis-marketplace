"""TMview adapter — the EUIPO-operated common database (tmdn.org/tmview).

TMview aggregates trademarks from EUIPO (office "EM"), the national offices
(incl. ÚPV — office "CZ") and WIPO ("WO"), so a single query covers both the
Czech and EU registers the watchdog cares about.

The public web app talks to a JSON backend. Exact field names have shifted
across TMview versions, so the parser is deliberately tolerant (it reads any of
several known aliases) and the endpoint + request shape are overridable via
CODEXIS_PLUGIN_ZNAMKY_TMVIEW_API_URL. The parser is unit-tested against a
recorded fixture; live endpoint/terms must be confirmed before production use.
"""

import os

from . import base

SOURCE = "TMVIEW"
DEFAULT_BASE = "https://www.tmdn.org/tmview/api"
DETAIL_BASE = "https://www.tmdn.org/tmview/welcome#/tmview/detail"

_FIELDS = [
    "ST13",
    "tmName",
    "applicantName",
    "tradeMarkOffice",
    "applicationDate",
    "applicationNumber",
    "markFeature",
    "niceClass",
    "viennaClass",
    "status",
    "markImageURI",
]


def _api_base() -> str:
    return (os.environ.get("CODEXIS_PLUGIN_ZNAMKY_TMVIEW_API_URL") or DEFAULT_BASE).rstrip("/")


def _first(record: dict, *names):
    for name in names:
        if name in record and record[name] not in (None, ""):
            return record[name]
    return ""


def _as_list(value) -> list:
    if value is None or value == "":
        return []
    if isinstance(value, list):
        return value
    if isinstance(value, str):
        # niceClass / viennaClass sometimes arrive comma- or semicolon-joined.
        return [p.strip() for p in value.replace(";", ",").split(",") if p.strip()]
    return [value]


def _detail_url(st13: str, office: str, number: str) -> str:
    if not st13:
        return ""
    return f"{DETAIL_BASE}/{st13}"


def parse_results(payload: dict) -> list:
    """Map a TMview search response into canonical candidates (tolerant)."""
    records = (
        payload.get("tradeMarks")
        or payload.get("results")
        or payload.get("trademarks")
        or []
    )
    candidates = []
    for rec in records:
        if not isinstance(rec, dict):
            continue
        st13 = str(_first(rec, "ST13", "st13", "stableId"))
        office = str(_first(rec, "tradeMarkOffice", "office", "officeCode"))
        number = str(_first(rec, "applicationNumber", "appNumber"))
        candidates.append(
            base.make_candidate(
                source=SOURCE,
                source_id=st13 or number,
                mark_text=_first(rec, "tmName", "markVerbalElement", "name"),
                mark_kind=_first(rec, "markFeature", "markKind", "feature"),
                applicant=_first(rec, "applicantName", "applicant", "holderName"),
                status=_first(rec, "status", "markCurrentStatusCode"),
                filing_date=_first(rec, "applicationDate", "filingDate"),
                nice_classes=_as_list(_first(rec, "niceClass", "niceClasses", "classDescriptionDetails")),
                vienna_codes=_as_list(_first(rec, "viennaClass", "viennaClasses", "viennaCode")),
                office=office,
                url_detail=_detail_url(st13, office, number),
                image_url=_first(rec, "markImageURI", "markImageUri", "imageUri"),
            )
        )
    return candidates


def search(query_text: str, nice_classes=None, offices=None, limit: int = 50) -> list:
    """Run a TMview basic search and return canonical candidates."""
    payload = {
        "page": "1",
        "pageSize": str(max(1, min(limit, 100))),
        "criteria": "C",
        "basicSearch": (query_text or "").strip(),
        "fOffices": [o.upper() for o in (offices or ["CZ", "EM", "WO"])],
        "fields": _FIELDS,
    }
    classes = base.normalize_classes(nice_classes)
    if classes:
        payload["fNiceClass"] = [str(c) for c in classes]
    response = base.post_json(
        f"{_api_base()}/search/results", payload, SOURCE,
        headers={"Referer": "https://www.tmdn.org/tmview/"},
    )
    return parse_results(response)


def fetch_image(candidate: dict):
    """Download a candidate's logo bytes, or None if it has no image."""
    url = (candidate or {}).get("image_url", "")
    if not url:
        return None
    try:
        return base.get_bytes(url, SOURCE)
    except Exception:  # noqa: BLE001 — image fetch is best-effort
        return None
