"""Shared core library for the sledovana-judikatura plugin.

Used by both the `cdx-sledovana-judikatura` CLI binary and the component
CGI handler.
"""

from . import notify, notify_settings, state, tracking
from .exceptions import (
    AmbiguousTopicPrefixError,
    AreaAlreadyExistsError,
    AreaIndexError,
    NoteIndexError,
    ReportNotFoundError,
    ReportSourceError,
    TopicError,
    TopicNotFoundError,
)

__all__ = [
    "AmbiguousTopicPrefixError",
    "AreaAlreadyExistsError",
    "AreaIndexError",
    "NoteIndexError",
    "ReportNotFoundError",
    "ReportSourceError",
    "TopicError",
    "TopicNotFoundError",
    "notify",
    "notify_settings",
    "state",
    "tracking",
]
