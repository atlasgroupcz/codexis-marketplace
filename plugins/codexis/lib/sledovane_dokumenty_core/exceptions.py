"""Typed exceptions for sledovane_dokumenty_core."""


class DocumentError(Exception):
    """Base exception for all sledovane-dokumenty errors."""


class DocumentNotTrackedError(DocumentError):
    """The requested document is not being tracked."""


class DocumentAlreadyTrackedError(DocumentError):
    """The requested document is already tracked."""


class DocumentNotFoundError(DocumentError):
    """The document does not exist in CODEXIS."""


class GroupNotFoundError(DocumentError):
    """The requested group does not exist."""


class RelatedTypeNotTrackedError(DocumentError):
    """The given relation type is not tracked for the document."""


class CdxClientError(DocumentError):
    """cdx-cli subprocess failed or returned an API-level error."""


class LlmDaemonError(DocumentError):
    """LLM daemon call failed or returned no result."""
