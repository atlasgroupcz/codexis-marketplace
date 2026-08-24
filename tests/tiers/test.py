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

# Lanes: sets of suites that can run at the same time as each other, because
# they reach for a different toolchain and a different part of the tree. Not
# "one pipeline stage" - that was the old grouping, and it existed only so
# `bun install` could happen between two of them. A pipeline may run the lanes
# in parallel or one after the other; either way each reports its own result.
LANES: dict[str, tuple[str, ...]] = {
    "backend": ("pytest", "cargo"),
    "frontend": ("frontend",),
}

# Derived, so the two cannot drift.
SUITES: tuple[str, ...] = tuple(name for lane in LANES.values() for name in lane)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run the repo's test suites.")
    parser.add_argument(
        "--lane",
        choices=sorted(LANES),
        help="Run one lane instead of all of them, for a pipeline that gives "
        "each its own stage.",
    )
    args = parser.parse_args(argv)
    wanted = LANES[args.lane] if args.lane else SUITES

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
