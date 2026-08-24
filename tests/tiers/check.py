"""The fast gate: lint, formatting, types and tests, across all three toolchains.

The marketplace test needs a live daemon, a VM and ~2 minutes. Everything here
needs none of that, so running it first turns a broken commit into a seconds-long
failure instead of a two-minute one. Reached as `tests/run.py check`.

Lint and typecheck belong here, not only the tests: a lint config that is broken
outright still leaves `turbo test` green, which is how a completely dead ESLint
setup once passed this gate.

Kept separate from the daemon tiers so it stays testable on its own
(`tests/unit/test_check_tier.py`) - nothing here talks to the daemon.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Callable
from pathlib import Path

from tests.harness.suites import (
    TURBO_TASKS,
    Suite,
    UnavailableToolchain,
    build_suite,
    cargo_available,
    default_availability,
    frontend_available,
    missing_pytest_message,
    pytest_available,
    ruff_available,
    run_gate,
    ty_available,
)

# Suite groups. `--suite <key>` runs one of them and refuses to skip a missing
# toolchain; so does `--strict` for all of them at once.
#
# Grouped by what they cover rather than by language: the plugin cores are being
# migrated from Python to Rust, so a split along toolchain lines would have to be
# renamed as that migration progresses.
SUITE_GROUPS: dict[str, tuple[str, ...]] = {
    # Plugin cores (pytest), their lint and format (ruff), and the CLI binaries
    # plus shared crates (cargo, cargo-fmt).
    "backend": (
        "ruff",
        "ruff-format",
        "ty",
        "conformance",
        "pytest",
        "cargo-fmt",
        "cargo",
    ),
    # The plugin component UIs: lint and types before the tests, because a
    # broken lint config passes `turbo test` without a word.
    "frontend": ("frontend-lint", "frontend-typecheck", "frontend"),
}


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
    enabled = {"pytest", "conformance"}
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
    return [build_suite(repo_root, name) for name in names]


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
        available = default_availability(repo_root)

    for name in names:
        if not available(name):
            raise UnavailableToolchain(
                f"check: suite {name!r} was requested, but its toolchain is "
                f"not installed.\n"
                f"  Refusing to skip it: a green stage that ran nothing is worse "
                f"than a red one.\n"
                f"  Install it on this machine (the CI agent provisions "
                f"uv/rustup/bun in the pipeline's Init stage)."
            )
    return [build_suite(repo_root, name) for name in names]


def all_suites_strict(
    repo_root: Path,
    available: Callable[[str], bool] | None = None,
) -> list[Suite]:
    """Every suite in every group, refusing to skip a missing toolchain.

    What `--suite <group>` promises for one stage, for a caller that runs the
    whole gate in one. [`select_suites`] skips what the machine lacks, which is
    right locally and wrong in a pipeline that provisions the toolchains: the
    stage would go green having run less than it was asked to.
    """
    return [
        suite
        for group in SUITE_GROUPS
        for suite in suites_for_group(repo_root, group, available)
    ]


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
        "check",
        select_suites(
            repo_root,
            cargo=has_cargo,
            frontend=has_frontend,
            ruff=has_ruff,
            ty=has_ty,
        ),
    )


def main(argv: list[str] | None = None) -> int:
    """CLI entry point.

    Bare invocation runs every suite the machine can, which is what you want
    locally. `--suite` and `--strict` both refuse to skip an absent toolchain -
    the pipeline provisions them, so a gap there is a broken agent, not
    something to skip past. They differ only in scope: one group, or all of
    them.
    """
    parser = argparse.ArgumentParser(description="Run the repo's check suites.")
    # Mutually exclusive: both say "this environment was provisioned, do not
    # skip", one per stage and one for the whole gate. Accepting both would
    # leave two ways to say the same thing.
    promise = parser.add_mutually_exclusive_group()
    promise.add_argument(
        "--suite",
        choices=sorted(SUITE_GROUPS),
        help="Run one group instead of auto-detecting (one per pipeline stage).",
    )
    promise.add_argument(
        "--strict",
        action="store_true",
        help="Run every suite and fail if any toolchain is missing, instead of "
        "skipping it. For CI, which provisions them.",
    )
    args = parser.parse_args(argv)
    repo_root = Path(__file__).resolve().parents[2]

    if args.suite is None and not args.strict:
        return run_default_gate(repo_root)

    if not pytest_available():
        print(missing_pytest_message(), file=sys.stderr, flush=True)
        return 1
    try:
        suites = (
            all_suites_strict(repo_root)
            if args.strict
            else suites_for_group(repo_root, args.suite)
        )
    except UnavailableToolchain as exc:
        print(exc, file=sys.stderr, flush=True)
        return 1
    return run_gate("check", suites)


if __name__ == "__main__":
    sys.exit(main())
