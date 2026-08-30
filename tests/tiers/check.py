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

# The parts a pipeline may ask for one at a time, so each gets its own stage and
# its own result. The split follows the toolchain: these two reach for different
# ones and write to different corners of the tree, which is what lets a pipeline
# run them at the same time if it wants to.
#
# Not the old suite grouping brought back. That existed so `bun install` could
# happen between two stages, which was a workaround and left with it.
PARTS: dict[str, tuple[str, ...]] = {
    "backend": ("ruff", "ruff-format", "ty", "conformance", "cargo-fmt"),
    "frontend": ("frontend-lint", "frontend-typecheck"),
}

# Derived, so the two cannot drift: a suite added to a part runs, and a suite
# added to none is a name this tier does not know.
SUITES: tuple[str, ...] = tuple(name for part in PARTS.values() for name in part)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run the repo's static checks.")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail when a toolchain is missing instead of skipping its suite. "
        "For CI, which provisions them.",
    )
    parser.add_argument(
        "--only",
        choices=sorted(PARTS),
        help="Run one part instead of all of them, for a pipeline that gives "
        "each its own stage.",
    )
    args = parser.parse_args(argv)

    wanted = PARTS[args.only] if args.only else SUITES

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
