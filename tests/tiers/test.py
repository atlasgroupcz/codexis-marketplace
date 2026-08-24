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

SUITES: tuple[str, ...] = ("pytest", "cargo", "frontend")


def main(argv: list[str] | None = None) -> int:
    if not pytest_available():
        print(missing_pytest_message(), file=sys.stderr, flush=True)
        return 1

    try:
        suites = select(REPO_ROOT, SUITES, strict=True)
    except UnavailableToolchain as exc:
        print(f"test: {exc}", file=sys.stderr, flush=True)
        return 1

    return run_gate("test", suites)


if __name__ == "__main__":
    sys.exit(main())
