"""Unit tests for the suite engine and the two tiers built on it."""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

import pytest

from tests.harness import suites
from tests.harness.suites import (
    Suite,
    UnavailableToolchain,
    missing_pytest_message,
    run_gate,
)
from tests.tiers import check as check_tier


class TestSelect:
    """`select` is the one place a tier says what it wants and gets what the
    environment can give, or an error saying it cannot."""

    ALL = staticmethod(lambda name: True)

    def test_strict_keeps_every_suite_asked_for_in_order(self):
        got = suites.select(
            Path("/repo"), ["ruff", "ty", "cargo"], strict=True, available=self.ALL
        )
        assert [s.name for s in got] == ["ruff", "ty", "cargo"]

    def test_strict_refuses_to_skip_a_missing_toolchain(self):
        """A green stage that ran nothing is worse than a red one."""
        with pytest.raises(UnavailableToolchain, match="cargo"):
            suites.select(
                Path("/repo"),
                ["ruff", "cargo"],
                strict=True,
                available=lambda n: n != "cargo",
            )

    def test_the_error_names_every_missing_suite_not_just_the_first(self):
        with pytest.raises(UnavailableToolchain, match=r"cargo.*frontend"):
            suites.select(
                Path("/repo"),
                ["cargo", "frontend"],
                strict=True,
                available=lambda n: False,
            )

    def test_without_strict_what_the_machine_lacks_is_left_out(self):
        """The laptop case: no Rust installed should not stop ruff running."""
        got = suites.select(
            Path("/repo"),
            ["ruff", "cargo"],
            strict=False,
            available=lambda n: n != "cargo",
        )
        assert [s.name for s in got] == ["ruff"]


class TestBuildSuite:
    def test_lint_and_format_are_separate_suites(self):
        """Run as one, the first failure would hide the second."""
        argv = {
            name: suites.build_suite(Path("/repo"), name).argv
            for name in ("ruff", "ruff-format", "ty")
        }
        # argv[0] is a resolved path - the project's copy when there is one.
        assert argv["ruff"][0].endswith("ruff")
        assert argv["ruff"][1:] == ["check"]
        assert argv["ruff-format"][1:] == ["format", "--check"]
        assert argv["ty"][1:] == ["check"]

    def test_project_copy_of_a_tool_wins_over_the_one_on_path(self, tmp_path):
        """The pinned versions in pyproject are the point: ty is pinned exactly
        because its diagnostics may change between any two releases. A stray
        copy on PATH must not decide the gate's verdict."""
        venv_ruff = tmp_path / ".venv" / "bin" / "ruff"
        venv_ruff.parent.mkdir(parents=True)
        venv_ruff.touch()
        assert suites._tool(tmp_path, "ruff") == str(venv_ruff)

    def test_a_tool_with_no_project_copy_falls_back_to_path(self, tmp_path):
        assert suites._tool(tmp_path, "ruff") == shutil.which("ruff")

    def test_frontend_suite_runs_in_the_frontend_directory(self):
        assert suites.build_suite(Path("/repo"), "frontend").cwd == (
            Path("/repo") / "frontend"
        )

    def test_an_unknown_suite_is_rejected(self):
        with pytest.raises(ValueError, match="sideways"):
            suites.build_suite(Path("/repo"), "sideways")


class TestMissingPytestMessage:
    def test_names_the_fix_so_a_ci_agent_can_be_repaired(self):
        """A bare ModuleNotFoundError doesn't say what to run."""
        msg = missing_pytest_message()
        assert "uv sync" in msg
        assert "uv run" in msg


class TestDefaultRunner:
    def test_cargo_runs_once_at_the_workspace_root(self, monkeypatch):
        """The repo is one Cargo workspace, so a single `cargo test` covers
        every member. Walking the crates by hand re-resolves the workspace once
        per crate and silently skips any member not on the hardcoded list.
        """
        calls = []

        class _Completed:
            returncode = 0

        monkeypatch.setattr(
            suites.subprocess,
            "run",
            lambda argv, cwd: calls.append((argv, cwd)) or _Completed(),
        )

        repo_root = Path(__file__).resolve().parents[2]
        suite = suites.build_suite(repo_root, "cargo")
        assert suites.default_runner(suite) == 0
        assert calls == [(suite.argv, repo_root)]


class TestRunGate:
    def test_returns_zero_when_every_suite_passes(self):
        suites = [Suite("a", ["true"], Path(".")), Suite("b", ["true"], Path("."))]
        assert run_gate("check", suites, runner=lambda s: 0) == 0

    def test_returns_one_when_any_suite_fails(self):
        suites = [Suite("a", ["true"], Path(".")), Suite("b", ["false"], Path("."))]
        assert (
            run_gate("check", suites, runner=lambda s: 0 if s.name == "a" else 1) == 1
        )

    def test_runs_every_suite_even_after_a_failure(self):
        """A failing pytest run must not hide a broken cargo suite."""
        seen = []

        def runner(suite):
            seen.append(suite.name)
            return 1

        suites = [Suite("a", ["x"], Path(".")), Suite("b", ["y"], Path("."))]
        run_gate("check", suites, runner=runner)
        assert seen == ["a", "b"]

    def test_empty_suite_list_passes(self):
        assert run_gate("check", [], runner=lambda s: 1) == 0


class TestConformanceSuite:
    """The repo's own conventions become a suite of their own, so a broken
    manifest reports under its own name instead of inside the pytest run."""

    def test_conformance_runs_only_the_conformance_tree(self):
        assert suites.build_suite(Path("/repo"), "conformance").argv[-1] == (
            "tests/conformance"
        )

    def test_pytest_skips_what_the_conformance_suite_already_ran(self):
        assert (
            "--ignore=tests/conformance"
            in suites.build_suite(Path("/repo"), "pytest").argv
        )

    def test_conformance_needs_nothing_pytest_does_not(self):
        probe = suites.default_availability(Path("/repo"))
        assert probe("conformance") == probe("pytest")


class TestExecutingTier:
    """`test` is the half that runs the code. While `check` still runs
    everything the two overlap; that is deliberate, so nothing is uncovered at
    any point between here and the narrowing."""

    def test_it_is_exactly_the_three_that_execute_something(self):
        from tests.tiers import test as test_tier

        assert test_tier.SUITES == ("pytest", "cargo", "frontend")

    def test_every_suite_it_names_can_be_built(self):
        from tests.tiers import test as test_tier

        for name in test_tier.SUITES:
            assert suites.build_suite(Path("/repo"), name).name == name

    def test_it_resolves_the_repo_root_not_the_tests_directory(self):
        from tests.tiers import test as test_tier

        assert (test_tier.REPO_ROOT / "pyproject.toml").is_file()


class TestTheSplit:
    """`check` narrows to the static half now that `test` runs the rest."""

    def test_the_static_tier_runs_nothing_that_executes_the_code(self):

        assert not {"pytest", "cargo", "frontend"} & set(check_tier.SUITES)

    def test_the_two_tiers_together_cover_every_suite_that_used_to_run(self):
        """The one that matters: it fails if narrowing dropped a suite instead
        of moving it, which would leave the gate green having run less."""
        from tests.tiers import test as test_tier

        was = {
            "ruff",
            "ruff-format",
            "ty",
            "conformance",
            "pytest",
            "cargo-fmt",
            "cargo",
            "frontend-lint",
            "frontend-typecheck",
            "frontend",
        }
        assert was == set(check_tier.SUITES) | set(test_tier.SUITES)

    def test_neither_tier_claims_a_suite_the_other_runs(self):
        from tests.tiers import test as test_tier

        assert not set(check_tier.SUITES) & set(test_tier.SUITES)

    def test_lint_runs_before_the_pytest_based_conformance_suite(self):
        """A syntax-level break should read as ruff's complaint, not as a
        collection error three suites later."""

        order = list(check_tier.SUITES)
        assert order.index("ruff") < order.index("conformance")


if __name__ == "__main__":
    sys.exit(pytest.main([__file__]))


if __name__ == "__main__":
    sys.exit(pytest.main([__file__]))


if __name__ == "__main__":
    sys.exit(pytest.main([__file__]))
