#!/usr/bin/env python3
"""The executing tier: run the code and see whether it works.

pytest, cargo test and turbo test. Everything here compiles or executes
something, which is what separates it from `check`: a red run means behaviour
changed, not that the source broke a rule.

Always strict, with no flag to relax it. There is nothing to auto-detect for: a
machine without cargo cannot tell you whether the Rust code works, so skipping
that suite and reporting green would be a lie.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from tests.harness.suites import (
    UnavailableToolchain,
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
    "backend": ("pytest", "cargo"),
    "frontend": ("frontend",),
}

# Derived, so the two cannot drift: a suite added to a part runs, and a suite
# added to none is a name this tier does not know.
SUITES: tuple[str, ...] = tuple(name for part in PARTS.values() for name in part)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run the repo's test suites.")
    parser.add_argument(
        "--only",
        choices=sorted(PARTS),
        help="Run one part instead of all of them, for a pipeline that gives "
        "each its own stage.",
    )
    args = parser.parse_args(argv)
    wanted = PARTS[args.only] if args.only else SUITES

    if not pytest_available():
        print(missing_pytest_message(), file=sys.stderr, flush=True)
        return 1

    try:
        suites = select(REPO_ROOT, wanted, strict=True)
    except UnavailableToolchain as exc:
        print(f"test: {exc}", file=sys.stderr, flush=True)
        return 1

    return run_gate("test", suites)


if __name__ == "__main__":
    sys.exit(main())
