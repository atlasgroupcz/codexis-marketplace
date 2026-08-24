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

# Lanes: sets of suites that can run at the same time as each other, because
# they reach for a different toolchain and a different part of the tree. Not
# "one pipeline stage" - that was the old grouping, and it existed only so
# `bun install` could happen between two of them. A pipeline may run the lanes
# in parallel or one after the other; either way each reports its own result.
LANES: dict[str, tuple[str, ...]] = {
    "backend": ("ruff", "ruff-format", "ty", "conformance", "cargo-fmt"),
    "frontend": ("frontend-lint", "frontend-typecheck"),
}

# Derived, so the two cannot drift: a suite added to a lane runs, and a suite
# added nowhere is a name this tier does not know.
SUITES: tuple[str, ...] = tuple(name for lane in LANES.values() for name in lane)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run the repo's static checks.")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail when a toolchain is missing instead of skipping its suite. "
        "For CI, which provisions them.",
    )
    parser.add_argument(
        "--lane",
        choices=sorted(LANES),
        help="Run one lane instead of all of them, for a pipeline that gives "
        "each its own stage.",
    )
    args = parser.parse_args(argv)

    wanted = LANES[args.lane] if args.lane else SUITES

    # conformance is pytest underneath, so this covers it too.
    if not pytest_available():
        print(missing_pytest_message(), file=sys.stderr, flush=True)
        return 1

    try:
        suites = select(
            REPO_ROOT,
            wanted,
            strict=args.strict,
            available=default_availability(REPO_ROOT),
        )
    except UnavailableToolchain as exc:
        print(f"check: {exc}", file=sys.stderr, flush=True)
        return 1

    return run_gate("check", suites)


if __name__ == "__main__":
    sys.exit(main())
