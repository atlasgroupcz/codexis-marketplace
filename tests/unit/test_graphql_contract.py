"""Unit tests for the cdxctl GraphQL contract helpers.

The contract itself - every cdxctl document validates against the daemon's
schema - needs a daemon and runs in the contract tier. What runs here is the
machinery that tier stands on: reading the documents out of the Rust source and
validating them against a schema.
"""

from __future__ import annotations

from graphql import build_schema, graphql_sync

from tests.harness.graphql_contract import (
    CDXCTL_GRAPHQL_SOURCE,
    Document,
    extract,
    introspection_query,
    schema_from_introspection,
    validate_documents,
)

SCHEMA = build_schema("""
    type Query {
        automations: [Automation!]!
    }

    type Automation {
        id: ID!
        title: String
    }
""")

VALID = "query Get { automations { id title } }"
# `marketplace` is the shape of every drift this check exists for: a field the
# client asks for and the schema does not have.
DRIFTED = "query Get { automations { marketplace } }"


def test_reads_a_document_included_from_a_graphql_file(tmp_path):
    (tmp_path / "queries").mkdir()
    (tmp_path / "queries" / "get.graphql").write_text(VALID)
    source = tmp_path / "graphql.rs"
    source.write_text('pub const GET: &str = include_str!("queries/get.graphql");\n')

    extraction = extract(source)

    assert [d.name for d in extraction.documents] == ["GET"]
    assert VALID in extraction.documents[0].body
    assert extraction.unreadable == []


def test_reads_a_declaration_rustfmt_wrapped_onto_the_next_line(tmp_path):
    # rustfmt breaks after the `=` once the line overruns its width, which is not
    # something the author chooses - a pattern anchored to one line silently
    # stops covering whichever document happens to have the longest name.
    (tmp_path / "queries").mkdir()
    (tmp_path / "queries" / "get.graphql").write_text(VALID)
    source = tmp_path / "graphql.rs"
    source.write_text(
        "pub const A_VERY_LONG_DOCUMENT_NAME: &str =\n"
        '    include_str!("queries/get.graphql");\n'
    )

    extraction = extract(source)

    assert [d.name for d in extraction.documents] == ["A_VERY_LONG_DOCUMENT_NAME"]
    assert extraction.unreadable == []


def test_reports_a_declaration_whose_body_it_cannot_read(tmp_path):
    # A document that escapes extraction would be reported as passing without
    # ever being validated. Naming it is the whole point of tracking this.
    source = tmp_path / "graphql.rs"
    source.write_text("pub const GET: &str = SOME_OTHER_CONST;\n")

    extraction = extract(source)

    assert extraction.documents == []
    assert extraction.unreadable == ["GET"]


def test_reports_an_include_whose_target_is_missing(tmp_path):
    source = tmp_path / "graphql.rs"
    source.write_text('pub const GET: &str = include_str!("queries/gone.graphql");\n')

    extraction = extract(source)

    assert extraction.documents == []
    assert extraction.unreadable == ["GET"]


def test_reports_a_field_the_schema_does_not_have():
    failures = validate_documents(SCHEMA, [Document("GET_AUTOMATIONS", DRIFTED)])

    assert [f.name for f in failures] == ["GET_AUTOMATIONS"]
    assert "marketplace" in failures[0].message


def test_accepts_a_document_the_schema_allows():
    assert validate_documents(SCHEMA, [Document("GET_AUTOMATIONS", VALID)]) == []


def test_reports_a_malformed_document_instead_of_raising():
    failures = validate_documents(
        SCHEMA, [Document("BROKEN", "query Get { automations {")]
    )

    assert [f.name for f in failures] == ["BROKEN"]


def test_a_schema_rebuilt_from_introspection_rejects_what_the_original_rejects():
    answer = graphql_sync(SCHEMA, introspection_query())
    assert answer.data is not None, answer.errors

    rebuilt = schema_from_introspection(answer.data)

    assert [
        f.name for f in validate_documents(rebuilt, [Document("GET", DRIFTED)])
    ] == ["GET"]
    assert validate_documents(rebuilt, [Document("GET", VALID)]) == []


def test_every_cdxctl_document_is_read_from_the_real_source():
    # Offline guard on the extractor itself: a document added in a shape it does
    # not know would never reach the daemon tier, and the tier would call that
    # a pass.
    extraction = extract(CDXCTL_GRAPHQL_SOURCE)

    assert extraction.unreadable == []
    assert extraction.documents
