"""Normalize raw ARES responses into Claude-friendly JSON shapes."""

from __future__ import annotations

from typing import Any


def company_summary(raw: dict[str, Any]) -> dict[str, Any]:
    """Return the compact identification shape used by `ares company`."""
    return {
        "source": "ares.basic",
        "raw": raw,
    }


def officers_summary(raw: dict[str, Any]) -> dict[str, Any]:
    """Return statutory-body and signing-authority data for `ares officers`."""
    return {
        "source": "ares.vr",
        "raw": raw,
    }


def trades_summary(raw: dict[str, Any]) -> dict[str, Any]:
    """Return trade-licence data for `ares trades`."""
    return {
        "source": "ares.rzp",
        "raw": raw,
    }


def owners_summary(raw: dict[str, Any]) -> dict[str, Any]:
    """Return beneficial-owner/compliance data for `ares owners`."""
    return {
        "source": "ares.rpsh",
        "raw": raw,
    }
