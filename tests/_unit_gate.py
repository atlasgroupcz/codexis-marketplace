"""The fast gate: lint, formatting, types and tests, across all three toolchains.

The marketplace test needs a live daemon, a VM and ~2 minutes. Everything here
needs none of that, so running it first turns a broken commit into a seconds-long
failure instead of a two-minute one. Reached as `tests/run.py check`.

Lint and typecheck belong here, not only the tests: a lint config that is broken
outright still leaves `turbo test` green, which is how a completely dead ESLint
setup once passed this gate.

Kept separate from `test-marketplace.py` so the gate stays testable on its own
(`tests/test_unit_gate.py`) - nothing here talks to the daemon.
"""

from __future__ import annotations

import argparse
import importlib.util
import shutil
import subprocess
import sys
from collections.abc import Callable, Iterable
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Suite:
    """One runnable suite: a display name, argv, and the cwd to run it in."""

    name: str
    argv: list[str]
    cwd: Path


# Suite groups, one per pipeline stage.
#
# These keys are a contract: Jenkins invokes this file as
# `tests/_unit_gate.py --suite <key>` from aws0's
# jenkins/dsl/cdx-daemon/marketplace-test-on-mr/Jenkinsfile. Renaming a key - or
# this file - is a two-repo change, which is why the tier was renamed to `check`
# and the filename was not. `tests/test_repo_hygiene.py` pins the key set.
#
# Grouped by what they cover rather than by language: the plugin cores are being
# migrated from Python to Rust, so a split along toolchain lines would have to be
# renamed as that migration progresses.
SUITE_GROUPS: dict[str, tuple[str, ...]] = {
    # Plugin cores (pytest), their lint and format (ruff), and the CLI binaries
    # plus shared crates (cargo, cargo-fmt).
    "backend": ("ruff", "ruff-format", "ty", "pytest", "cargo-fmt", "cargo"),
    # The plugin component UIs: lint and types before the tests, because a
    # broken lint config passes `turbo test` without a word.
    "frontend": ("frontend-lint", "frontend-typecheck", "frontend"),
}


# Frontend suite -> the turbo task it runs. A table rather than three branches:
# the only thing that differs between them is this word.
TURBO_TASKS: dict[str, str] = {
    "frontend-lint": "lint",
    "frontend-typecheck": "typecheck",
    "frontend": "test",
}


class UnavailableToolchain(RuntimeError):
    """A suite was explicitly requested but its toolchain isn't installed."""


def pytest_available() -> bool:
    """Whether pytest is importable by the interpreter that will run it."""
    return importlib.util.find_spec("pytest") is not None


def missing_pytest_message() -> str:
    """Actionable error for an agent that has no pytest.

    Unlike cargo and the JS toolchain, pytest is NOT optional - skipping it
    would leave the gate green while testing nothing. Fail loudly, and say
    exactly how to repair the agent.
    """
    return (
        "check: pytest is not installed for this interpreter.\n"
        "  The suites cannot run, and skipping them would report a green gate "
        "that tested nothing.\n"
        "  Fix: uv sync   then run this via   uv run python3 tests/run.py check\n"
        "  (test-only deps live in pyproject.toml under [dependency-groups].dev)"
    )


def _tool(repo_root: Path, name: str) -> str | None:
    """Locate a Python dev tool, preferring the project's own copy.

    ruff and ty ship as binaries with no `python -m` entry point, so the trick
    that finds pytest through the running interpreter does not work for them.

    The project's venv comes first because the versions in pyproject are the
    point: ty is pinned exactly, since it is pre-1.0 and its diagnostics may
    change between any two versions. A different copy on PATH would decide the
    gate's verdict instead.
    """
    in_venv = repo_root / ".venv" / "bin" / name
    if in_venv.is_file():
        return str(in_venv)
    return shutil.which(name)


def _require_tool(repo_root: Path, name: str) -> str:
    """The tool's path, or raise.

    `_build_suite` is only reached once availability has been confirmed, so a
    miss here is the caller skipping that check - a bug, not a machine without
    the toolchain.
    """
    path = _tool(repo_root, name)
    if path is None:
        raise UnavailableToolchain(f"check: {name} was not found")
    return path


def ruff_available(repo_root: Path) -> bool:
    """Whether ruff can be found for this repo."""
    return _tool(repo_root, "ruff") is not None


def ty_available(repo_root: Path) -> bool:
    """Whether ty can be found for this repo."""
    return _tool(repo_root, "ty") is not None


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
    if name == "ruff":
        return Suite("ruff", [_require_tool(repo_root, "ruff"), "check"], repo_root)
    if name == "ruff-format":
        # Separate from `ruff check`: the two answer different questions, and
        # run as one suite the first failure would hide the second.
        return Suite(
            "ruff-format",
            [_require_tool(repo_root, "ruff"), "format", "--check"],
            repo_root,
        )
    if name == "ty":
        return Suite("ty", [_require_tool(repo_root, "ty"), "check"], repo_root)
    if name == "cargo-fmt":
        # Separate from `cargo test` for the same reason ruff-format is separate
        # from ruff check: a formatting complaint must not hide a failing test.
        return Suite(
            "cargo-fmt",
            ["cargo", "fmt", "--all", "--check"],
            repo_root,
        )
    if name == "cargo":
        # One workspace at the repo root covers the shared crates and every
        # plugin CLI, so this is the whole Rust tree.
        return Suite(
            "cargo",
            ["cargo", "test", "--all-features", "-q"],
            repo_root,
        )
    if name in TURBO_TASKS:
        runner = "bun" if shutil.which("bun") else "npx"
        argv = [runner, "turbo", TURBO_TASKS[name]]
        return Suite(name, argv, repo_root / "frontend")
    raise ValueError(f"unknown suite: {name}")


def select_suites(
    repo_root: Path,
    cargo: bool,
    frontend: bool,
    ruff: bool = True,
    ty: bool = True,
) -> list[Suite]:
    """The suites runnable in this environment, in the order they should run.

    ruff first: it takes a fraction of a second and reads every file, so a
    syntax-level break shows up as itself rather than as a pytest collection
    error. ty next, then pytest - the broadest suite, and the only remaining
    one that needs no compile or install step.
    """
    enabled = {"pytest"}
    if ruff:
        enabled |= {"ruff", "ruff-format"}
    if ty:
        enabled.add("ty")
    if cargo:
        enabled |= {"cargo", "cargo-fmt"}
    if frontend:
        enabled |= set(TURBO_TASKS)

    # Order comes from SUITE_GROUPS rather than being restated here: a suite
    # added to a group but forgotten in this function would run in CI and
    # silently not run locally, which is the false-green this module prevents.
    names = [n for group in SUITE_GROUPS.values() for n in group if n in enabled]
    return [_build_suite(repo_root, name) for name in names]


def _default_runner(suite: Suite) -> int:
    return subprocess.run(suite.argv, cwd=suite.cwd).returncode


def _default_availability(repo_root: Path) -> Callable[[str], bool]:
    """Probe the real environment for a suite's toolchain."""

    def probe(name: str) -> bool:
        if name == "pytest":
            return pytest_available()
        if name in ("ruff", "ruff-format"):
            return ruff_available(repo_root)
        if name == "ty":
            return ty_available(repo_root)
        if name in ("cargo", "cargo-fmt"):
            return cargo_available()
        if name in TURBO_TASKS:
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
    provisioned. Skipping here would report a green stage that ran nothing -
    exactly the false-green this gate exists to prevent - so a missing
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
                f"check: suite {name!r} was requested via --suite {group}, "
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

    Every suite runs even after one fails - a broken pytest run must not hide a
    broken cargo run, otherwise the next push just trades one red for another.
    """
    failed: list[str] = []
    for suite in suites:
        print(f"\n─── check: {suite.name} ───", flush=True)
        if runner(suite) != 0:
            failed.append(suite.name)

    if failed:
        print(f"\ncheck FAILED: {', '.join(failed)}", file=sys.stderr, flush=True)
        return 1
    return 0


def run_default_gate(repo_root: Path) -> int:
    """Select and run the suites appropriate for this environment."""
    if not pytest_available():
        print(missing_pytest_message(), file=sys.stderr, flush=True)
        return 1

    has_cargo = cargo_available()
    has_frontend = frontend_available(repo_root)
    has_ruff = ruff_available(repo_root)
    has_ty = ty_available(repo_root)
    if not has_cargo:
        print("check: no cargo on PATH - skipping Rust suites", flush=True)
    if not has_frontend:
        print(
            "check: no bun/npx or frontend/node_modules - skipping frontend suites",
            flush=True,
        )
    if not has_ruff:
        print("check: no ruff - skipping lint and format", flush=True)
    if not has_ty:
        print("check: no ty - skipping the type check", flush=True)
    return run_gate(
        select_suites(
            repo_root,
            cargo=has_cargo,
            frontend=has_frontend,
            ruff=has_ruff,
            ty=has_ty,
        )
    )


def main(argv: list[str] | None = None) -> int:
    """CLI entry point.

    Bare invocation runs every suite the machine can, which is what you want
    locally. `--suite` runs exactly one group and fails when its toolchain is
    absent - the pipeline provisions them, so a gap there is a broken agent,
    not something to skip past.
    """
    parser = argparse.ArgumentParser(description="Run the repo's check suites.")
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
