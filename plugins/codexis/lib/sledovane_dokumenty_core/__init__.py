"""Shared core library for the sledovane-dokumenty plugin.

Used by both the `cdx-sledovane-dokumenty` CLI binary and the component
CGI handler. All business logic (CODEXIS fetching, diff generation, state
management, change detection) lives here so neither caller duplicates it.
"""

from . import clients, diff, related, state, tracking
from .exceptions import (
    CdxClientError,
    DocumentAlreadyTrackedError,
    DocumentError,
    DocumentNotFoundError,
    DocumentNotTrackedError,
    GroupNotFoundError,
    LlmDaemonError,
    RelatedTypeNotTrackedError,
)

__all__ = [
    "clients",
    "diff",
    "related",
    "state",
    "tracking",
    "CdxClientError",
    "DocumentAlreadyTrackedError",
    "DocumentError",
    "DocumentNotFoundError",
    "DocumentNotTrackedError",
    "GroupNotFoundError",
    "LlmDaemonError",
    "RelatedTypeNotTrackedError",
]
