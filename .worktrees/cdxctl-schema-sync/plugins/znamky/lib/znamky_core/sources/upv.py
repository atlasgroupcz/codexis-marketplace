"""ÚPV direct adapter — optional CZ enrichment on top of TMview.

ÚPV (Úřad průmyslového vlastnictví) is a TMview participant, so Czech marks are
already retrieved through TMview's "CZ" office filter. This adapter is an
optional *direct* source for richer CZ-only detail (Vienna codes, status), and
stays disabled unless CODEXIS_PLUGIN_ZNAMKY_UPV_URL is configured — we don't
ship a brittle HTML scraper that would silently rot. When enabled, the endpoint
is expected to return the same JSON record shape TMview's parser understands.
"""

import os

from . import base, tmview

SOURCE = "UPV"


def enabled() -> bool:
    return bool(os.environ.get("CODEXIS_PLUGIN_ZNAMKY_UPV_URL"))


def search(query_text: str, nice_classes=None, limit: int = 50) -> list:
    if not enabled():
        return []
    url = os.environ["CODEXIS_PLUGIN_ZNAMKY_UPV_URL"].rstrip("/")
    payload = {
        "query": (query_text or "").strip(),
        "niceClasses": [str(c) for c in base.normalize_classes(nice_classes)],
        "limit": max(1, min(limit, 100)),
    }
    response = base.post_json(f"{url}/search", payload, SOURCE)
    candidates = tmview.parse_results(response)
    for candidate in candidates:
        candidate["source"] = SOURCE
        candidate["office"] = candidate.get("office") or "CZ"
        candidate["territory"] = "CZ"
    return candidates
