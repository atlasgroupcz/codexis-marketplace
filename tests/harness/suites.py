"""Building and running one named check suite.

A suite is a command, a working directory, and a name the log prints. This
module knows how to build each one by name, whether its toolchain is present,
and how to run a list of them; which suites belong together is a tier's
business, not this module's.
"""

from __future__ import annotations

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

    `build_suite` is only reached once availability has been confirmed, so a
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


def build_suite(repo_root: Path, name: str) -> Suite:
    """One suite by name, independent of whether its toolchain is present."""
    if name == "pytest":
        # --ignore, because the conformance suite below already ran that tree.
        return Suite(
            "pytest",
            [sys.executable, "-m", "pytest", "-q", "--ignore=tests/conformance"],
            repo_root,
        )
    if name == "conformance":
        # The repo's own conventions: manifests, READMEs, i18n, hygiene. Its own
        # suite rather than part of pytest, because a broken manifest and a
        # broken function are different failures and deserve different names.
        return Suite(
            "conformance",
            [sys.executable, "-m", "pytest", "-q", "tests/conformance"],
            repo_root,
        )
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


def default_runner(suite: Suite) -> int:
    return subprocess.run(suite.argv, cwd=suite.cwd).returncode


def default_availability(repo_root: Path) -> Callable[[str], bool]:
    """Probe the real environment for a suite's toolchain."""

    def probe(name: str) -> bool:
        if name in ("pytest", "conformance"):
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


def run_gate(
    tier: str,
    suites: Iterable[Suite],
    runner: Callable[[Suite], int] = default_runner,
) -> int:
    """Run every suite and return 0 only if all passed.

    Every suite runs even after one fails - a broken pytest run must not hide a
    broken cargo run, otherwise the next push just trades one red for another.

    `tier` only names the run in the output, but it has to be passed rather than
    assumed: more than one tier runs suites now, and a log that says `check`
    while `test` is running sends a reader to the wrong half.
    """
    failed: list[str] = []
    for suite in suites:
        print(f"\n─── {tier}: {suite.name} ───", flush=True)
        if runner(suite) != 0:
            failed.append(suite.name)

    if failed:
        print(f"\n{tier} FAILED: {', '.join(failed)}", file=sys.stderr, flush=True)
        return 1
    return 0
