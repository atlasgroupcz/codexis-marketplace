"""Custom exceptions for znamky_core (trademark watchdog)."""


class ZnamkyError(Exception):
    """Base exception for all trademark-watchdog errors."""


class ApiNetworkError(ZnamkyError):
    """Network error talking to a public trademark source (TMview / ÚPV / EUIPO)."""


class ApiHttpError(ZnamkyError):
    """A trademark source returned an unexpected HTTP error."""

    def __init__(self, status: int, message: str = ""):
        super().__init__(message or f"HTTP {status}")
        self.status = status


class ApiParseError(ZnamkyError):
    """A trademark source response could not be parsed into the expected shape."""


class InvalidMarkError(ZnamkyError):
    """Watched-mark input is invalid (missing text/logo, bad Nice class, …)."""


class MarkAlreadyTrackedError(ZnamkyError):
    """An equivalent mark is already being watched."""


class MarkNotTrackedError(ZnamkyError):
    """The mark is not in the local watch state."""


class CredentialsMissingError(ZnamkyError):
    """No source credentials configured — the user must set them up first."""


class CredentialsInvalidError(ZnamkyError):
    """The configured source credentials were rejected by the provider."""
