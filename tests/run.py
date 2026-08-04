#!/usr/bin/env python3
"""Single front door for the marketplace test suites.

Usage:
  python tests/run.py unit          # fast unit gate (pytest + cargo + frontend)
  python tests/run.py marketplace   # daemon install/marketplace acceptance
  python tests/run.py eval          # promptfoo behavioral evals (the one tier)
  python tests/run.py all           # every tier in order, fail-fast on the gate

Anything after the subcommand is passed straight through to the underlying
script, so `python tests/run.py eval --only ares` works. This is a thin
dispatcher; each tier's real logic stays in its own module.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

TESTS_DIR = Path(__file__).resolve().parent

# subcommand -> script path (relative to tests/), invoked as `python <script> ...`.
COMMANDS: dict[str, str] = {
    "unit": "_unit_gate.py",
    "marketplace": "test-marketplace.py",
    "eval": "promptfoo/run.py",
}

# Order for `all`: the tiers run in COMMANDS order - the unit gate is defined first
# because it needs no daemon and fails fastest.
ORDER: list[str] = list(COMMANDS)


def build_argv(command: str, passthrough: list[str]) -> list[str]:
    """The full argv for one subcommand, including passthrough args."""
    return [sys.executable, str(TESTS_DIR / COMMANDS[command]), *passthrough]


def _run_one(command: str, passthrough: list[str]) -> int:
    argv = build_argv(command, passthrough)
    print(f"\n=== tests/run.py: {command} ===", flush=True)
    return subprocess.run(argv).returncode


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    if not args or args[0] in ("-h", "--help"):
        print(__doc__)
        return 0

    command, passthrough = args[0], args[1:]

    if command == "all":
        # `all` fans out to every tier in ORDER, stopping at the first failure so
        # a broken unit gate does not burn ~2 minutes on the daemon suites. The
        # unit gate takes no daemon args and its argparse rejects unknown flags, so
        # passthrough (--daemon/--token/...) goes only to the daemon-backed tiers.
        for name in ORDER:
            code = _run_one(name, [] if name == "unit" else passthrough)
            if code != 0:
                print(f"tests/run.py: {name} failed - stopping.", file=sys.stderr)
                return code
        return 0

    if command not in COMMANDS:
        print(
            f"tests/run.py: unknown command {command!r}; "
            f"known: {', '.join(sorted(COMMANDS))}, all",
            file=sys.stderr,
        )
        return 2

    return _run_one(command, passthrough)


if __name__ == "__main__":
    sys.exit(main())
