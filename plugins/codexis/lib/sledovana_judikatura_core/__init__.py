"""Shared core library for the sledovana-judikatura plugin.

Used by both the `cdx-sledovana-judikatura` CLI binary and the component
CGI handler.
"""

from . import state, tracking
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
    "state",
    "tracking",
    "AmbiguousTopicPrefixError",
    "AreaAlreadyExistsError",
    "AreaIndexError",
    "NoteIndexError",
    "ReportNotFoundError",
    "ReportSourceError",
    "TopicError",
    "TopicNotFoundError",
]
