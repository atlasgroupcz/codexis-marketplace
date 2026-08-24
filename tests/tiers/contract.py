#!/usr/bin/env python3
"""
cdxctl GraphQL Contract Test

Holds every GraphQL document cdxctl sends against the schema a live daemon
serves. One read-only introspection request, no mutations, seconds to run.

Its own tier rather than a step of the marketplace test: this has nothing to do
with installing plugins, so a red run here means one thing only - the client
drifted from the daemon - and it says so before the two-minute lifecycle suite
starts.

Usage:
  python3 tests/run.py contract --daemon URL [--token JWT]

Example:
  python3 tests/run.py contract --daemon http://localhost:8086
"""

from __future__ import annotations

import argparse
import os
import sys

from tests.harness.daemon_client import DaemonClient
from tests.harness.graphql_contract import (
    CDXCTL_GRAPHQL_SOURCE,
    extract,
    introspection_query,
    schema_from_introspection,
    validate_documents,
)
from tests.harness.output import Results
from tests.harness.preflight import ensure_ready


def check_contract(client: DaemonClient, r: Results) -> None:
    """Report every way cdxctl's documents disagree with the daemon's schema."""
    r.section("cdxctl GraphQL contract")

    extraction = extract(CDXCTL_GRAPHQL_SOURCE)
    for name in extraction.unreadable:
        r.fail(f"{name}: declared as a document but its body could not be read")

    try:
        payload = client.gql_data(introspection_query())
    except (RuntimeError, OSError) as exc:
        # Everything that stops the schema arriving lands here as one line rather
        # than a traceback: a GraphQL-level refusal (introspection turned off, an
        # expired token) raises RuntimeError, an unreachable or 404 endpoint
        # raises urllib's HTTPError/URLError, which are OSError. Not having the
        # schema is a different report from a document that disagrees with it.
        r.fail(f"could not fetch the daemon schema: {exc}")
        return

    failures = validate_documents(
        schema_from_introspection(payload), extraction.documents
    )
    for failure in failures:
        r.fail(f"{failure.name}: {failure.message}")

    if not failures and not extraction.unreadable:
        r.ok(f"{len(extraction.documents)} documents match the daemon schema")


def main() -> int:
    parser = argparse.ArgumentParser(description="cdxctl GraphQL Contract Test")
    parser.add_argument(
        "--daemon", required=True, help="Daemon URL (e.g. http://localhost:8086)"
    )
    parser.add_argument(
        "--token",
        default=os.environ.get("CDX_DAEMON_TOKEN", ""),
        help="JWT token. Defaults to $CDX_DAEMON_TOKEN, which is how CI "
        "should pass it: an argument is visible in the process "
        "list and in any shell trace.",
    )
    args = parser.parse_args()
    if not args.token:
        parser.error("a token is required: pass --token or set CDX_DAEMON_TOKEN")

    client = DaemonClient(args.daemon, args.token)
    r = Results()
    r.log(f"Daemon: {args.daemon}")

    # Without this a cold boot ends in "could not fetch the daemon schema",
    # which is true and the wrong thing to fix.
    try:
        ensure_ready(client, r)
    except RuntimeError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    check_contract(client, r)
    return r.summary()


if __name__ == "__main__":
    sys.exit(main())
