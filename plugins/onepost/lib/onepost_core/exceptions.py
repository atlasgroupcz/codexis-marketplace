"""Custom exceptions for ONEPOST API access."""


class OnepostError(Exception):
    """Base exception for ONEPOST plugin errors."""


class ConfigError(OnepostError):
    """Configuration is invalid."""


class ApiKeyMissingError(OnepostError):
    """API key is not configured."""


class ApiAuthError(OnepostError):
    """API key was rejected."""


class ApiPermissionError(OnepostError):
    """API key or user cannot access the requested operation."""


class ApiRateLimitError(OnepostError):
    """ONEPOST rate limit was exceeded."""


class ApiNetworkError(OnepostError):
    """Network error talking to ONEPOST."""


class ApiHttpError(OnepostError):
    """ONEPOST returned an unexpected HTTP error."""

    def __init__(self, status: int, message: str, code: str | None = None):
        super().__init__(message)
        self.status = status
        self.code = code
