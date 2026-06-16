"""Demo source — a small bundled dataset of sample trademarks.

Enabled by the CODEXIS_PLUGIN_ZNAMKY_DEMO env flag. Lets the similarity matching
be exercised end to end (scoring, tiers, donut, collision timeline) in
environments where the live public registers are not reachable. The scoring +
threshold layer decides which of these become collisions, exactly like a real
source — only the candidate list is local instead of fetched.
"""

import json
import os
from pathlib import Path

from . import base

SOURCE = "DEMO"
_PATH = Path(__file__).resolve().parent / "demo_marks.json"


def enabled() -> bool:
    return bool(os.environ.get("CODEXIS_PLUGIN_ZNAMKY_DEMO"))


def _load() -> list:
    try:
        with open(_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, list) else []
    except (OSError, json.JSONDecodeError):
        return []


def search(query_text=None, nice_classes=None, limit: int = 100) -> list:
    """Return all demo marks as canonical candidates; scoring filters them."""
    out = []
    for m in _load():
        out.append(
            base.make_candidate(
                source=SOURCE,
                source_id=m.get("id", ""),
                mark_text=m.get("name", ""),
                mark_kind=m.get("kind", "word"),
                applicant=m.get("applicant", ""),
                status=m.get("status", ""),
                filing_date=m.get("filing_date", ""),
                nice_classes=m.get("nice_classes", []),
                vienna_codes=m.get("vienna_codes", []),
                office=m.get("office", "CZ"),
                url_detail=m.get("url", ""),
                image_url=m.get("image_url", ""),
            )
        )
    return out[: max(1, limit)]
