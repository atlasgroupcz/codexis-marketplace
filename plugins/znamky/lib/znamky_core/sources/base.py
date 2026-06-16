"""Shared helpers for trademark source adapters: HTTP transport + canonical model.

Every adapter (TMview, ÚPV, …) normalises its hits into the same *candidate*
dict so the scoring/tracking layers never care where a mark came from:

    {
      "source": "TMVIEW"|"UPV"|"EUIPO",
      "source_id": str,            # stable per-source id (TMview ST13, app. no.)
      "mark_text": str,            # verbal element ("" for pure figurative)
      "mark_kind": "word"|"figurative"|"combined"|"other",
      "applicant": str,
      "status": str,
      "filing_date": str,          # ISO-ish or ""
      "nice_classes": [int, ...],
      "vienna_codes": [str, ...],
      "office": str,               # "CZ","EM"(EUIPO),"WO"(WIPO), …
      "territory": "CZ"|"EU"|"INT",
      "url_detail": str,
      "image_url": str,            # logo URL ("" if none)
    }
"""

import http.client
import json
import time
import urllib.error
import urllib.request

from ..exceptions import ApiHttpError, ApiNetworkError, ApiParseError

USER_AGENT = "cdx-znamky-watchdog/1.0 (+https://atlasgroup.cz)"
TIMEOUT = 30
RETRIES = 2

# Office code → coarse territory bucket used for relevance/labelling.
_OFFICE_TERRITORY = {
    "CZ": "CZ",
    "EM": "EU",   # EUIPO (EU trademark)
    "WO": "INT",  # WIPO international registration
}

_KIND_MAP = {
    "word": "word",
    "wordmark": "word",
    "figurative": "figurative",
    "figure": "figurative",
    "combined": "combined",
    "stylizedcharacters": "combined",
    "stylised": "combined",
}


def office_to_territory(office: str) -> str:
    return _OFFICE_TERRITORY.get((office or "").upper(), "INT")


def normalize_kind(value: str) -> str:
    key = (value or "").strip().lower().replace(" ", "").replace("-", "")
    return _KIND_MAP.get(key, "other" if value else "word")


def normalize_classes(values) -> list:
    out = []
    for v in values or []:
        text = str(v).strip()
        if text.isdigit():
            out.append(int(text))
    return sorted(set(out))


def make_candidate(**fields) -> dict:
    office = (fields.get("office") or "").upper()
    return {
        "source": fields.get("source", ""),
        "source_id": str(fields.get("source_id", "")),
        "mark_text": (fields.get("mark_text") or "").strip(),
        "mark_kind": normalize_kind(fields.get("mark_kind", "")),
        "applicant": (fields.get("applicant") or "").strip(),
        "status": (fields.get("status") or "").strip(),
        "filing_date": (fields.get("filing_date") or "").strip(),
        "nice_classes": normalize_classes(fields.get("nice_classes")),
        "vienna_codes": [str(c).strip() for c in (fields.get("vienna_codes") or []) if str(c).strip()],
        "office": office,
        "territory": fields.get("territory") or office_to_territory(office),
        "url_detail": (fields.get("url_detail") or "").strip(),
        "image_url": (fields.get("image_url") or "").strip(),
    }


def candidate_key(candidate: dict) -> tuple:
    return (candidate.get("source", ""), candidate.get("source_id", ""))


def dedupe(candidates: list) -> list:
    seen = set()
    out = []
    for c in candidates:
        key = candidate_key(c)
        if key in seen:
            continue
        seen.add(key)
        out.append(c)
    return out


# ── HTTP transport (stdlib only) ─────────────────────────────────────────────


def _request(req: urllib.request.Request, source_name: str) -> bytes:
    last_err = None
    for attempt in range(RETRIES + 1):
        try:
            with urllib.request.urlopen(req, timeout=TIMEOUT) as resp:
                return resp.read()
        except urllib.error.HTTPError as exc:
            if exc.code >= 500 and attempt < RETRIES:
                last_err = exc
                time.sleep(1 + attempt)
                continue
            body = exc.read().decode("utf-8", "replace")[:300]
            raise ApiHttpError(exc.code, f"{source_name} vrátilo HTTP {exc.code}: {body}")
        except (urllib.error.URLError, http.client.HTTPException, OSError) as exc:
            # URLError, connection resets ([Errno 104]), broken TLS, DNS, timeouts —
            # all "source unreachable". Retry, then degrade to ApiNetworkError so the
            # caller (search_all) captures it instead of crashing the whole request.
            if attempt < RETRIES:
                last_err = exc
                time.sleep(1 + attempt)
                continue
            reason = getattr(exc, "reason", None) or exc
            raise ApiNetworkError(f"Nepodařilo se kontaktovat {source_name}: {reason}")
    raise ApiNetworkError(f"{source_name} nedostupné: {last_err}")


def post_json(url: str, payload: dict, source_name: str, headers: dict = None) -> dict:
    data = json.dumps(payload).encode("utf-8")
    base_headers = {
        "Content-Type": "application/json",
        "Accept": "application/json",
        "User-Agent": USER_AGENT,
    }
    base_headers.update(headers or {})
    req = urllib.request.Request(url, data=data, headers=base_headers, method="POST")
    raw = _request(req, source_name)
    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        raise ApiParseError(f"Neplatná JSON odpověď {source_name}: {exc}")


def get_bytes(url: str, source_name: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT}, method="GET")
    return _request(req, source_name)


def get_json(url: str, source_name: str, headers: dict = None) -> dict:
    base_headers = {"Accept": "application/json", "User-Agent": USER_AGENT}
    base_headers.update(headers or {})
    req = urllib.request.Request(url, headers=base_headers, method="GET")
    raw = _request(req, source_name)
    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        raise ApiParseError(f"Neplatná JSON odpověď {source_name}: {exc}")
