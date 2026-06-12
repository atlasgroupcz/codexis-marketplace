"""Command-level ARES operations."""

from __future__ import annotations

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
        body = {"obchodniJmeno": normalized_query, "pocet": limit, "start": 0}
        raw = self.client.post_json(endpoint, body)
        return formatters.search_summary(
            raw,
            echo={
                "command": "ares search",
                "endpoint": f"POST {endpoint}",
                "query": normalized_query,
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
