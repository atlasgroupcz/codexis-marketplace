"""Command-level ARES operations."""

from __future__ import annotations

import re
import unicodedata
from typing import Any

from .client import AresClient
from .errors import AresCliError
from . import formatters
from .sources import RAW_SOURCES, SourceSpec


class AresService:
    """Maps legal-user commands to ARES source calls."""

    def __init__(self, client: AresClient | None = None) -> None:
        self.client = client or AresClient()

    def search(self, query: str, *, limit: int) -> dict[str, Any]:
        normalized_query = query.strip()
        if not normalized_query:
            raise AresCliError("Vyhledávací dotaz nesmí být prázdný.")
        if not 1 <= limit <= 1000:
            raise AresCliError("--limit musí být v rozsahu 1 až 1000.")

        endpoint = "/ekonomicke-subjekty/vyhledat"
        fetch_limit = min(1000, max(limit, 50))
        body = {"obchodniJmeno": normalized_query, "pocet": fetch_limit, "start": 0}
        raw = self.client.post_json(endpoint, body)
        ranked = _rank_search_candidates(raw.get("ekonomickeSubjekty") or [], normalized_query)
        ranked_raw = {**raw, "ekonomickeSubjekty": ranked[:limit]}
        return formatters.search_summary(
            ranked_raw,
            echo={
                "command": "ares search",
                "endpoint": f"POST {endpoint}",
                "query": normalized_query,
                "limit": limit,
                "fetchLimit": fetch_limit,
                "ranking": "local_relevance",
                "requestBody": body,
            },
        )

    def company(self, ico: str) -> dict[str, Any]:
        normalized_ico = _normalize_ico(ico)
        source = RAW_SOURCES["basic"]
        raw = self._fetch_source(normalized_ico, source)
        return formatters.company_summary(
            raw,
            echo={
                "command": "ares company",
                "endpoint": f"GET {source.path_template}",
                "ico": normalized_ico,
            },
        )

    def officers(self, ico: str) -> dict[str, Any]:
        normalized_ico = _normalize_ico(ico)
        source = RAW_SOURCES["vr"]
        raw = self._fetch_source(normalized_ico, source)
        return formatters.officers_summary(
            raw,
            echo={
                "command": "ares officers",
                "endpoint": f"GET {source.path_template}",
                "ico": normalized_ico,
            },
        )

    def trades(self, ico: str) -> dict[str, Any]:
        normalized_ico = _normalize_ico(ico)
        source = RAW_SOURCES["rzp"]
        raw = self._fetch_source(normalized_ico, source)
        return formatters.trades_summary(
            raw,
            echo={
                "command": "ares trades",
                "endpoint": f"GET {source.path_template}",
                "ico": normalized_ico,
            },
        )

    def raw(self, ico: str, *, source: str) -> dict[str, Any]:
        normalized_ico = _normalize_ico(ico)
        spec = RAW_SOURCES[source]
        return self._fetch_source(normalized_ico, spec)

    def _fetch_source(self, ico: str, source: SourceSpec) -> dict[str, Any]:
        return self.client.get_json(source.path_template.format(ico=ico))


def _normalize_ico(ico: str) -> str:
    digits = "".join(ch for ch in ico if ch.isdigit())
    if not 1 <= len(digits) <= 8:
        raise AresCliError("IČO musí obsahovat 1 až 8 číslic.")
    return digits.zfill(8)


def _rank_search_candidates(candidates: list[dict[str, Any]], query: str) -> list[dict[str, Any]]:
    indexed = list(enumerate(candidates))
    return [
        candidate
        for _, candidate in sorted(
            indexed,
            key=lambda item: (-_search_score(item[1], query), item[0]),
        )
    ]


def _search_score(candidate: dict[str, Any], query: str) -> int:
    name = str(candidate.get("obchodniJmeno") or "")
    normalized_query = _normalize_search_text(query)
    normalized_name = _normalize_search_text(name)
    core_name = _strip_legal_suffix(normalized_name)
    tokens = normalized_name.split()
    score = 0

    if normalized_name == normalized_query:
        score += 1000
    if core_name == normalized_query:
        score += 950
    if tokens[: len(normalized_query.split())] == normalized_query.split():
        score += 220
    if normalized_query in tokens:
        score += 120
    elif normalized_query and normalized_query in normalized_name:
        score += 40

    registrations = candidate.get("seznamRegistraci") or {}
    if isinstance(registrations, dict):
        if registrations.get("stavZdrojeVr") == "AKTIVNI":
            score += 20
        if registrations.get("stavZdrojeRzp") == "AKTIVNI":
            score += 5

    if str(candidate.get("pravniForma") or "") in {"112", "121"}:
        score += 10

    score -= min(100, max(0, len(normalized_name) - len(normalized_query)))
    return score


def _normalize_search_text(value: str) -> str:
    decomposed = unicodedata.normalize("NFKD", value)
    ascii_text = "".join(ch for ch in decomposed if not unicodedata.combining(ch))
    lowered = ascii_text.lower()
    return re.sub(r"\s+", " ", re.sub(r"[^a-z0-9]+", " ", lowered)).strip()


def _strip_legal_suffix(normalized_name: str) -> str:
    suffixes = (
        " a s",
        " s r o",
        " spol s r o",
        " v o s",
        " k s",
        " z s",
        " p o",
    )
    result = normalized_name
    changed = True
    while changed:
        changed = False
        for suffix in suffixes:
            if result.endswith(suffix):
                result = result[: -len(suffix)].strip()
                changed = True
    return result
