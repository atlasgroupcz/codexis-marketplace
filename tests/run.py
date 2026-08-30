#!/usr/bin/env python3
"""Single front door for the marketplace test suites.

Usage:
  python tests/run.py check         # static: lint, format, types, repo conventions
  python tests/run.py test          # executing: pytest, cargo test, turbo test
  python tests/run.py contract      # GraphQL documents vs a live daemon's schema
  python tests/run.py marketplace   # plugin install/uninstall acceptance
  python tests/run.py eval          # promptfoo behavioral evals
  python tests/run.py all           # every tier in order, stopping at the first failure

Each tier runs as a subprocess rather than an import. That keeps the contract to
an exit code, lets a tier call sys.exit() without taking the dispatcher with it,
and stops one tier's failure from ending the rest of an `all` run. Tiers stay
runnable on their own: `python -m tests.tiers.check` works from the repo root.

Anything after the subcommand is passed through to the tier, so
`python tests/run.py eval --only ares` works. Only the daemon-backed tiers get
it; the static two reject an unknown flag rather than ignoring it.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

TESTS_DIR = Path(__file__).resolve().parent

REPO_ROOT = TESTS_DIR.parent

# subcommand -> module path, invoked as `python -m <module>` from the repo root.
# Not a file path: a script run by path gets its own directory on sys.path and
# not the root, which every tier used to work around by hand.
COMMANDS: dict[str, str] = {
    "check": "tests.tiers.check",
    "test": "tests.tiers.test",
    "contract": "tests.tiers.contract",
    "marketplace": "tests.tiers.marketplace",
    "eval": "tests.promptfoo.run",
}

# Order for `all`: the tiers run in COMMANDS order, cheapest first. `check` needs
# no daemon at all; `contract` needs one but answers in a single request, so a
# client that drifted from the schema is reported before the install lifecycle
# spends two minutes finding out the same thing the hard way.
ORDER: list[str] = list(COMMANDS)

# The tiers that talk to a daemon, and so the only ones the passthrough
# arguments (--daemon, --token, --git-url) mean anything to. The static tiers'
# argparse rejects an unknown flag rather than ignoring it.
DAEMON_TIERS = frozenset({"contract", "marketplace", "eval"})


def build_argv(command: str, passthrough: list[str]) -> list[str]:
    """The full argv for one subcommand, including passthrough args."""
    return [sys.executable, "-m", COMMANDS[command], *passthrough]


def _run_one(command: str, passthrough: list[str]) -> int:
    argv = build_argv(command, passthrough)
    print(f"\n=== tests/run.py: {command} ===", flush=True)
    return subprocess.run(argv, cwd=REPO_ROOT).returncode


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    if not args or args[0] in ("-h", "--help"):
        print(__doc__)
        return 0

    command, passthrough = args[0], args[1:]

    if command == "all":
        # `all` fans out to every tier in ORDER, stopping at the first failure so
        # a broken gate does not burn ~2 minutes on the daemon suites.
        for name in ORDER:
            code = _run_one(name, passthrough if name in DAEMON_TIERS else [])
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
