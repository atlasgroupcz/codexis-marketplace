"""Custom exceptions for katastr_core."""


class KatastrError(Exception):
    """Base exception for all katastr errors."""


class ApiKeyMissingError(KatastrError):
    """API key is not configured."""


class ApiKeyInvalidError(KatastrError):
    """API key is invalid (rejected by ČÚZK)."""


class ApiNetworkError(KatastrError):
    """Network error talking to ČÚZK API."""


class ApiHttpError(KatastrError):
    """ČÚZK API returned an unexpected HTTP error."""

    def __init__(self, status: int, message: str = ""):
        super().__init__(message or f"HTTP {status}")
        self.status = status


class InvalidProceedingNumberError(KatastrError):
    """Proceeding number doesn't match expected format."""


class ProceedingNotFoundError(KatastrError):
    """Proceeding doesn't exist in ČÚZK."""


class ProceedingNotTrackedError(KatastrError):
    """Proceeding is not in the local tracking state."""


class ProceedingAlreadyTrackedError(KatastrError):
    """Proceeding is already being tracked."""
