#!/usr/bin/env python3
"""The executing tier: run the code and see whether it works.

pytest, cargo test and turbo test. Everything here compiles or executes
something, which is what separates it from `check`: a red run means behaviour
changed, not that the source broke a rule.

A missing toolchain is refused rather than skipped. This tier has no local-first
mode because there is nothing to auto-detect for: a machine without cargo cannot
tell you whether the Rust code works, and reporting green would be a lie.
"""

from __future__ import annotations

import sys
from pathlib import Path

from tests.harness.suites import (
    UnavailableToolchain,
    build_suite,
    default_availability,
    missing_pytest_message,
    pytest_available,
    run_gate,
)

REPO_ROOT = Path(__file__).resolve().parents[2]

SUITES: tuple[str, ...] = ("pytest", "cargo", "frontend")


def main(argv: list[str] | None = None) -> int:
    if not pytest_available():
        print(missing_pytest_message(), file=sys.stderr, flush=True)
        return 1

    available = default_availability(REPO_ROOT)
    missing = [name for name in SUITES if not available(name)]
    if missing:
        print(
            UnavailableToolchain(
                f"test: {', '.join(repr(n) for n in missing)} cannot run here, "
                f"their toolchains are not installed.\n"
                f"  Refusing to skip them: a green run that ran nothing is worse "
                f"than a red one.\n"
                f"  Install them on this machine (the CI agent provisions "
                f"uv/rustup/bun in the pipeline's Init stage)."
            ),
            file=sys.stderr,
            flush=True,
        )
        return 1

    return run_gate("test", [build_suite(REPO_ROOT, name) for name in SUITES])


if __name__ == "__main__":
    sys.exit(main())
