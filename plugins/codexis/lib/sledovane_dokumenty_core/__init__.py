"""Shared core library for the sledovane-dokumenty plugin.

Used by both the `cdx-sledovane-dokumenty` CLI binary and the component
CGI handler. All business logic (CODEXIS fetching, diff generation, state
management, change detection) lives here so neither caller duplicates it.
"""

from . import (
    clients,
    diff,
    harvest,
    notify,
    project_check,
    projects,
    related,
    state,
    tracking,
)
from .exceptions import (
    CdxClientError,
    DocumentAlreadyTrackedError,
    DocumentError,
    DocumentNotFoundError,
    DocumentNotTrackedError,
    GroupNotFoundError,
    LlmDaemonError,
    RelatedTypeNotTrackedError,
)
from .projects import WatchedProjectError

__all__ = [
    "CdxClientError",
    "DocumentAlreadyTrackedError",
    "DocumentError",
    "DocumentNotFoundError",
    "DocumentNotTrackedError",
    "GroupNotFoundError",
    "LlmDaemonError",
    "RelatedTypeNotTrackedError",
    "WatchedProjectError",
    "clients",
    "diff",
    "harvest",
    "notify",
    "project_check",
    "projects",
    "related",
    "state",
    "tracking",
]
