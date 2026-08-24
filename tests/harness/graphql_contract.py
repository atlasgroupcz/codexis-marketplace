"""Hold cdxctl's GraphQL documents against the daemon's schema.

cdxctl talks to the daemon through 36 GraphQL documents that live beside its
Rust source as .graphql files. Nothing has ever compared them to the schema they are
written against, so a field the daemon renames stays in the client until a user
runs the command and gets a validation error back. That has happened three
times: `marketplace` and `plugin` on Agent and Skill, `agentFullName` on
Automation, and `action` on Notification.

This module is the machinery: read the documents out of the Rust source, turn a
daemon introspection response into a schema, and validate one against the other.
The tier that owns a daemon calls it - see tests/tiers/contract.py.

What it does NOT see: the variables. `mutation($input: CreateNotificationInput!)`
is a legal document whatever the client later puts in `$input`, so a dropped
*input* field passes here and fails at the daemon. Closing that needs the
request body as it goes on the wire, which is cdxctl's own `stub_daemon`.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import cast

from graphql import (
    GraphQLSchema,
    GraphQLSyntaxError,
    IntrospectionQuery,
    build_client_schema,
    get_introspection_query,
    parse,
    validate,
)

REPO_ROOT = Path(__file__).resolve().parents[2]
CDXCTL_GRAPHQL_SOURCE = REPO_ROOT / "plugins/codexis/src/cdxctl/src/graphql.rs"

# The declaration is the index, not the content: it names the .graphql file and,
# by being an include_str!, proves that file reaches the binary. A file nothing
# includes ships nowhere and is not this check's business.
#
# `\s*` after the `=`: rustfmt wraps a declaration that overruns the line width
# onto the next line, and a pattern anchored to one line reports it as unreadable.
_INCLUDED = re.compile(r'pub const ([A-Z0-9_]+): &str =\s*include_str!\("([^"]+)"\);')
# Every declaration, whatever its body. What this finds and the two above do not
# is a document that would never be validated - see Extraction.unreadable.
_DECLARED = re.compile(r"pub const ([A-Z0-9_]+): &str")


@dataclass(frozen=True)
class Document:
    """One named GraphQL document as cdxctl sends it."""

    name: str
    body: str


@dataclass(frozen=True)
class Failure:
    name: str
    message: str


@dataclass(frozen=True)
class Extraction:
    documents: list[Document]
    #: Declared as a document but its body could not be read. Never empty
    #: silently: an unread document is one this check cannot speak for.
    unreadable: list[str]


def extract(rust_file: Path) -> Extraction:
    """Every GraphQL document declared in `rust_file`, plus what it could not read."""
    source = rust_file.read_text()
    documents: list[Document] = []
    for name, relative in _INCLUDED.findall(source):
        try:
            documents.append(Document(name, (rust_file.parent / relative).read_text()))
        except OSError:
            continue  # falls into `unreadable` below, which is reported as a failure
    read = {document.name for document in documents}
    return Extraction(
        documents=sorted(documents, key=lambda document: document.name),
        unreadable=[name for name in _DECLARED.findall(source) if name not in read],
    )


def introspection_query() -> str:
    """The read-only query that asks a daemon for its schema.

    Descriptions are left out: they are most of the payload and validation never
    looks at them.
    """
    return get_introspection_query(descriptions=False)


def schema_from_introspection(payload: dict) -> GraphQLSchema:
    """Build a schema from what `introspection_query()` returned."""
    # The daemon answers with plain JSON and graphql-core describes the same
    # shape as a TypedDict. Nothing checks the payload before the cast: a body
    # that is not an introspection result fails inside build_client_schema, and
    # its message says so better than a guard here could.
    return build_client_schema(cast(IntrospectionQuery, payload))


def validate_documents(
    schema: GraphQLSchema, documents: list[Document]
) -> list[Failure]:
    """Every way the given documents disagree with the schema.

    A document that will not even parse is reported like any other failure
    rather than raised, so one broken document does not hide the rest.
    """
    failures: list[Failure] = []
    for document in documents:
        try:
            errors = validate(schema, parse(document.body))
        except GraphQLSyntaxError as exc:
            failures.append(Failure(document.name, exc.message))
            continue
        failures.extend(Failure(document.name, error.message) for error in errors)
    return failures
