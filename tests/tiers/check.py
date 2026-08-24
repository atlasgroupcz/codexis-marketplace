#!/usr/bin/env python3
"""The static tier: everything that can fail without running the code.

Lint, formatting, types, and the repository's own conventions. Nothing here
executes a plugin or a test, so a red run means the source broke a rule rather
than that behaviour changed. What runs the code is the `test` tier.

Order: ruff first, because it reads every file in a fraction of a second, so a
syntax-level break reads as itself rather than as a collection error three
suites later. Conformance comes after ty for the same reason, being pytest: it
needs the tree to parse.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from tests.harness.suites import (
    UnavailableToolchain,
    default_availability,
    missing_pytest_message,
    pytest_available,
    run_gate,
    select,
)

REPO_ROOT = Path(__file__).resolve().parents[2]

SUITES: tuple[str, ...] = (
    "ruff",
    "ruff-format",
    "ty",
    "conformance",
    "cargo-fmt",
    "frontend-lint",
    "frontend-typecheck",
)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run the repo's static checks.")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail when a toolchain is missing instead of skipping its suite. "
        "For CI, which provisions them.",
    )
    args = parser.parse_args(argv)

    # conformance is pytest underneath, so this covers it too.
    if not pytest_available():
        print(missing_pytest_message(), file=sys.stderr, flush=True)
        return 1

    try:
        suites = select(
            REPO_ROOT,
            SUITES,
            strict=args.strict,
            available=default_availability(REPO_ROOT),
        )
    except UnavailableToolchain as exc:
        print(f"check: {exc}", file=sys.stderr, flush=True)
        return 1

    return run_gate("check", suites)


if __name__ == "__main__":
    sys.exit(main())
