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
        raise AresCliError(
            "Search is scaffolded. Intended mapping: POST /ekonomicke-subjekty/vyhledat "
            "with business-name criteria, then normalize candidates."
        )

    def company(self, ico: str) -> dict[str, Any]:
        raw = self._fetch_source(ico, RAW_SOURCES["basic"])
        return formatters.company_summary(raw)

    def officers(self, ico: str) -> dict[str, Any]:
        raw = self._fetch_source(ico, RAW_SOURCES["vr"])
        return formatters.officers_summary(raw)

    def trades(self, ico: str) -> dict[str, Any]:
        raw = self._fetch_source(ico, RAW_SOURCES["rzp"])
        return formatters.trades_summary(raw)

    def owners(self, ico: str) -> dict[str, Any]:
        raw = self._fetch_source(ico, RAW_SOURCES["rpsh"])
        return formatters.owners_summary(raw)

    def raw(self, ico: str, *, source: str) -> dict[str, Any]:
        spec = RAW_SOURCES[source]
        return self._fetch_source(ico, spec)

    def _fetch_source(self, ico: str, source: SourceSpec) -> dict[str, Any]:
        normalized_ico = _normalize_ico(ico)
        return self.client.get_json(source.path_template.format(ico=normalized_ico))


def _normalize_ico(ico: str) -> str:
    digits = "".join(ch for ch in ico if ch.isdigit())
    if not 1 <= len(digits) <= 8:
        raise AresCliError("IČO musí obsahovat 1 až 8 číslic.")
    return digits.zfill(8)
