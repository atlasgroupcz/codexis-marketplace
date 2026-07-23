"""Shared core library for the katastr plugin.

Used by both the `katastr` CLI binary and the HTTP backend serving the UI.
All heavy logic (ČÚZK API calls, state management, change detection) lives
here so neither caller has to duplicate it.
"""

from . import api_client, settings, tracking
from .exceptions import (
    ApiHttpError,
    ApiKeyInvalidError,
    ApiKeyMissingError,
    ApiNetworkError,
    InvalidProceedingNumberError,
    KatastrError,
    ProceedingAlreadyTrackedError,
    ProceedingNotFoundError,
    ProceedingNotTrackedError,
)

__all__ = [
    "api_client",
    "settings",
    "tracking",
    "KatastrError",
    "ApiKeyMissingError",
    "ApiKeyInvalidError",
    "ApiHttpError",
    "ApiNetworkError",
    "InvalidProceedingNumberError",
    "ProceedingNotFoundError",
    "ProceedingNotTrackedError",
    "ProceedingAlreadyTrackedError",
]
