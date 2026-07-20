"""Fast local unit suites, run as a gate before the daemon integration test.

The marketplace test needs a live daemon, a VM and ~2 minutes. The repo's own
unit tests need none of that and finish in seconds, so running them first turns
a broken commit into a 6-second failure instead of a 2-minute one.

Kept separate from `test-marketplace.py` so the gate stays testable on its own
(`tests/test_unit_gate.py`) — nothing here talks to the daemon.
"""

from __future__ import annotations

import argparse
import importlib.util
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterable


@dataclass(frozen=True)
class Suite:
    """One runnable unit suite: a display name, argv, and the cwd to run it in."""

    name: str
    argv: list[str]
    cwd: Path


# Shared Rust crates. Plugin crates are deliberately excluded: each is a separate
# Cargo project and compiling all 22 from cold would cost more than the daemon
# test this gate is meant to front-run.
_RUST_CRATES = ("cdx-common", "cdx-notify", "cdx-cgi", "cdx-watch")

# Suite groups, one per pipeline stage. Grouped by what they cover rather than by
# language: the plugin cores are being migrated from Python to Rust, so a split
# along toolchain lines would have to be renamed as that migration progresses.
SUITE_GROUPS: dict[str, tuple[str, ...]] = {
    # Plugin cores (pytest) and the CLI binaries plus shared crates (cargo).
    "backend": ("pytest", "cargo"),
    # The plugin component UIs.
    "frontend": ("frontend",),
}


class UnavailableToolchain(RuntimeError):
    """A suite was explicitly requested but its toolchain isn't installed."""


def pytest_available() -> bool:
    """Whether pytest is importable by the interpreter that will run it."""
    return importlib.util.find_spec("pytest") is not None


def missing_pytest_message() -> str:
    """Actionable error for an agent that has no pytest.

    Unlike cargo and the JS toolchain, pytest is NOT optional — skipping it
    would leave the gate green while testing nothing. Fail loudly, and say
    exactly how to repair the agent.
    """
    return (
        "unit gate: pytest is not installed for this interpreter.\n"
        "  The suites cannot run, and skipping them would report a green gate "
        "that tested nothing.\n"
        "  Fix: uv sync   then run this via   uv run python3 tests/_unit_gate.py\n"
        "  (test-only deps live in pyproject.toml under [dependency-groups].dev)"
    )


def cargo_available() -> bool:
    """Whether a Rust toolchain is on PATH. CI agents may not have one."""
    return shutil.which("cargo") is not None


def frontend_available(repo_root: Path) -> bool:
    """Whether the frontend suite can run: a runner on PATH and installed deps.

    `turbo test` without `node_modules` fails on a missing binary rather than a
    failing test, which would read as a red suite for the wrong reason.
    """
    has_runner = shutil.which("bun") is not None or shutil.which("npx") is not None
    return has_runner and (repo_root / "frontend" / "node_modules").is_dir()


def _build_suite(repo_root: Path, name: str) -> Suite:
    """One suite by name, independent of whether its toolchain is present."""
    if name == "pytest":
        return Suite("pytest", [sys.executable, "-m", "pytest", "-q"], repo_root)
    if name == "cargo":
        return Suite("cargo", ["cargo", "test", "--all-features", "-q"],
                     repo_root / "shared" / "rust")
    if name == "frontend":
        runner = "bun" if shutil.which("bun") else "npx"
        return Suite("frontend", [runner, "turbo", "test"], repo_root / "frontend")
    raise ValueError(f"unknown suite: {name}")


def select_suites(repo_root: Path, cargo: bool, frontend: bool) -> list[Suite]:
    """The suites runnable in this environment, in the order they should run.

    pytest first: it is the broadest suite and the only one that needs no
    compile or install step, so it surfaces the most breakage soonest.
    """
    names = ["pytest"]
    if cargo:
        names.append("cargo")
    if frontend:
        names.append("frontend")
    return [_build_suite(repo_root, name) for name in names]


def _default_runner(suite: Suite) -> int:
    if suite.name == "cargo":
        # No workspace at shared/rust — run each crate in turn.
        worst = 0
        for crate in _RUST_CRATES:
            crate_dir = suite.cwd / crate
            if not crate_dir.is_dir():
                continue
            worst = max(worst, subprocess.run(suite.argv, cwd=crate_dir).returncode)
        return worst
    return subprocess.run(suite.argv, cwd=suite.cwd).returncode


def _default_availability(repo_root: Path) -> Callable[[str], bool]:
    """Probe the real environment for a suite's toolchain."""
    def probe(name: str) -> bool:
        if name == "pytest":
            return pytest_available()
        if name == "cargo":
            return cargo_available()
        if name == "frontend":
            return frontend_available(repo_root)
        raise ValueError(f"unknown suite: {name}")

    return probe


def suites_for_group(
    repo_root: Path,
    group: str,
    available: Callable[[str], bool] | None = None,
) -> list[Suite]:
    """The suites in `group`, or raise if any of their toolchains is missing.

    Unlike [`select_suites`], which skips what the machine lacks, an explicit
    group request is a promise from the caller that the environment was
    provisioned. Skipping here would report a green stage that ran nothing —
    exactly the false-green this gate exists to prevent — so a missing
    toolchain is an error the pipeline must surface.
    """
    names = SUITE_GROUPS.get(group)
    if names is None:
        raise ValueError(
            f"unknown suite group {group!r}; known: {', '.join(sorted(SUITE_GROUPS))}"
        )

    if available is None:
        available = _default_availability(repo_root)

    for name in names:
        if not available(name):
            raise UnavailableToolchain(
                f"unit gate: suite {name!r} was requested via --suite {group}, "
                f"but its toolchain is not installed.\n"
                f"  Refusing to skip it: a green stage that ran nothing is worse "
                f"than a red one.\n"
                f"  Install it on this machine (the CI agent provisions "
                f"uv/rustup/bun in the pipeline's Init stage)."
            )
    return [_build_suite(repo_root, name) for name in names]


def run_gate(
    suites: Iterable[Suite],
    runner: Callable[[Suite], int] = _default_runner,
) -> int:
    """Run every suite and return 0 only if all passed.

    Every suite runs even after one fails — a broken pytest run must not hide a
    broken cargo run, otherwise the next push just trades one red for another.
    """
    failed: list[str] = []
    for suite in suites:
        print(f"\n─── unit gate: {suite.name} ───", flush=True)
        if runner(suite) != 0:
            failed.append(suite.name)

    if failed:
        print(f"\nUnit gate FAILED: {', '.join(failed)}", file=sys.stderr, flush=True)
        return 1
    return 0


def run_default_gate(repo_root: Path) -> int:
    """Select and run the suites appropriate for this environment."""
    if not pytest_available():
        print(missing_pytest_message(), file=sys.stderr, flush=True)
        return 1

    has_cargo = cargo_available()
    has_frontend = frontend_available(repo_root)
    if not has_cargo:
        print("unit gate: no cargo on PATH — skipping Rust suites", flush=True)
    if not has_frontend:
        print(
            "unit gate: no bun/npx or frontend/node_modules — skipping frontend suite",
            flush=True,
        )
    return run_gate(select_suites(repo_root, cargo=has_cargo, frontend=has_frontend))


def main(argv: list[str] | None = None) -> int:
    """CLI entry point.

    Bare invocation runs every suite the machine can, which is what you want
    locally. `--suite` runs exactly one group and fails when its toolchain is
    absent — the pipeline provisions them, so a gap there is a broken agent,
    not something to skip past.
    """
    parser = argparse.ArgumentParser(description="Run the repo's unit suites.")
    parser.add_argument(
        "--suite",
        choices=sorted(SUITE_GROUPS),
        help="Run one group instead of auto-detecting (one per pipeline stage).",
    )
    args = parser.parse_args(argv)
    repo_root = Path(__file__).resolve().parent.parent

    if args.suite is None:
        return run_default_gate(repo_root)

    if not pytest_available():
        print(missing_pytest_message(), file=sys.stderr, flush=True)
        return 1
    try:
        suites = suites_for_group(repo_root, args.suite)
    except UnavailableToolchain as exc:
        print(exc, file=sys.stderr, flush=True)
        return 1
    return run_gate(suites)


if __name__ == "__main__":
    sys.exit(main())
