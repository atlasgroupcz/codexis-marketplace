"""Typed exceptions for sledovana_judikatura_core."""


class TopicError(Exception):
    """Base exception for all sledovana-judikatura errors."""


class TopicNotFoundError(TopicError):
    """The requested topic does not exist."""


class AmbiguousTopicPrefixError(TopicError):
    """A partial UUID matches more than one topic."""


class AreaIndexError(TopicError):
    """Area index out of range."""


class NoteIndexError(TopicError):
    """Note index out of range."""


class AreaAlreadyExistsError(TopicError):
    """An area with the same name already exists on the topic."""


class ReportNotFoundError(TopicError):
    """The requested report does not exist for the topic."""


class ReportSourceError(TopicError):
    """The provided report source (file / stdin) could not be read or parsed."""
