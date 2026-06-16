"""Shared core library for the znamky (trademark watchdog) plugin.

Used by both the `znamky-cli` binary and the HTTP backend serving the UI. All
heavy logic (public-register search, similarity scoring, watch-state tracking,
change detection) lives here so neither caller duplicates it.
"""

from . import image_similarity, scoring, sources, text_similarity, tracking
from .exceptions import (
    ApiHttpError,
    ApiNetworkError,
    ApiParseError,
    InvalidMarkError,
    MarkAlreadyTrackedError,
    MarkNotTrackedError,
    ZnamkyError,
)

__all__ = [
    "image_similarity",
    "scoring",
    "sources",
    "text_similarity",
    "tracking",
    "ZnamkyError",
    "ApiNetworkError",
    "ApiHttpError",
    "ApiParseError",
    "InvalidMarkError",
    "MarkAlreadyTrackedError",
    "MarkNotTrackedError",
]
