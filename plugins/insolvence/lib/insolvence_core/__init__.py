"""Shared core library for the insolvence plugin.

Used by both the `insolvence-cli` binary and the HTTP backend serving the UI.
All heavy logic (ISIR lookups, subject tracking state, change detection) lives
here so neither caller has to duplicate it.
"""

from . import api_client, tracking
from .exceptions import (
    ApiHttpError,
    ApiNetworkError,
    ApiParseError,
    InsolvenceError,
    InvalidSubjectError,
    SubjectAlreadyTrackedError,
    SubjectNotTrackedError,
)

__all__ = [
    "api_client",
    "tracking",
    "InsolvenceError",
    "ApiNetworkError",
    "ApiHttpError",
    "ApiParseError",
    "InvalidSubjectError",
    "SubjectAlreadyTrackedError",
    "SubjectNotTrackedError",
]
