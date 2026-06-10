"""Typed output contracts for future parser work."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class EntityRef:
    """Minimal canonical reference to a Czech economic entity."""

    ico: str
    name: str


@dataclass(frozen=True)
class SourceRecord:
    """ARES source metadata carried with each normalized response."""

    source: str
    fetched_at: str | None = None
