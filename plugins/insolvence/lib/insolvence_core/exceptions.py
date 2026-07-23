"""Custom exceptions for insolvence_core."""


class InsolvenceError(Exception):
    """Base exception for all insolvence errors."""


class ApiNetworkError(InsolvenceError):
    """Network error talking to the ISIR public interface."""


class ApiHttpError(InsolvenceError):
    """ISIR returned an unexpected HTTP error."""

    def __init__(self, status: int, message: str = ""):
        super().__init__(message or f"HTTP {status}")
        self.status = status


class ApiParseError(InsolvenceError):
    """The ISIR response could not be parsed into the expected shape."""


class InvalidSubjectError(InsolvenceError):
    """Subject input is invalid (bad IČO, or missing name/date of birth)."""


class SubjectAlreadyTrackedError(InsolvenceError):
    """Subject is already being tracked."""


class SubjectNotTrackedError(InsolvenceError):
    """Subject is not in the local tracking state."""
