"""Unit tests for the pre-daemon check gate."""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

import pytest

from tests.harness import suites
from tests.tiers import check as check_tier
from tests.tiers.check import (
    SUITE_GROUPS,
    Suite,
    UnavailableToolchain,
    missing_pytest_message,
    run_gate,
    select_suites,
    suites_for_group,
)


class TestSuiteGroups:
    """The groups map 1:1 onto the pipeline stages that invoke them."""

    def test_backend_group_covers_lint_plugin_cores_and_binaries(self):
        assert SUITE_GROUPS["backend"] == (
            "ruff",
            "ruff-format",
            "ty",
            "conformance",
            "pytest",
            "cargo-fmt",
            "cargo",
        )

    def test_frontend_group_lints_and_typechecks_before_it_tests(self):
        assert SUITE_GROUPS["frontend"] == (
            "frontend-lint",
            "frontend-typecheck",
            "frontend",
        )


class TestSuitesForGroup:
    """Selection only - toolchain presence is stubbed so these stay hermetic."""

    ALL_PRESENT = staticmethod(lambda _name: True)

    def test_backend_selects_lint_pytest_and_cargo_only(self):
        suites = suites_for_group(Path("/repo"), "backend", available=self.ALL_PRESENT)
        assert [s.name for s in suites] == [
            "ruff",
            "ruff-format",
            "ty",
            "conformance",
            "pytest",
            "cargo-fmt",
            "cargo",
        ]

    def test_frontend_selects_lint_typecheck_and_tests(self):
        suites = suites_for_group(Path("/repo"), "frontend", available=self.ALL_PRESENT)
        assert [s.name for s in suites] == [
            "frontend-lint",
            "frontend-typecheck",
            "frontend",
        ]

    def test_cargo_suite_runs_over_the_whole_workspace(self):
        """One workspace at the repo root, so the plugin CLIs are covered too."""
        suites = suites_for_group(Path("/repo"), "backend", available=self.ALL_PRESENT)
        cargo = next(s for s in suites if s.name == "cargo")
        assert cargo.cwd == Path("/repo")

    def test_formatting_is_its_own_suite_so_it_cannot_hide_a_failing_test(self):
        suites = suites_for_group(Path("/repo"), "backend", available=self.ALL_PRESENT)
        fmt = next(s for s in suites if s.name == "cargo-fmt")
        assert fmt.argv == ["cargo", "fmt", "--all", "--check"]

    def test_unknown_group_is_rejected(self):
        with pytest.raises(ValueError, match="unknown suite group"):
            suites_for_group(Path("/repo"), "sideways", available=self.ALL_PRESENT)

    def test_missing_toolchain_raises_instead_of_silently_skipping(self):
        """An explicitly requested group must never report success on a
        toolchain the environment lacks - that is the false-green the gate
        exists to prevent. Auto-detection may skip; an explicit request may not.
        """
        with pytest.raises(UnavailableToolchain, match="cargo"):
            suites_for_group(
                Path("/repo"), "backend", available=lambda name: name != "cargo"
            )


class TestMissingPytestMessage:
    def test_names_the_fix_so_a_ci_agent_can_be_repaired(self):
        """A bare ModuleNotFoundError doesn't say what to run."""
        msg = missing_pytest_message()
        assert "uv sync" in msg
        assert "uv run" in msg


class TestStrictGate:
    """`--strict` is the pipeline's promise that the agent was provisioned.

    A bare run skips what the machine lacks, which is right locally and wrong in
    CI: a stage that ran less than it should still reports green. `--suite`
    already carried that promise per stage; `--strict` carries it for a caller
    that runs the whole gate in one.
    """

    def test_strict_runs_every_suite_in_every_group(self):
        suites = check_tier.all_suites_strict(Path("/repo"), available=lambda n: True)
        names = [suite.name for suite in suites]
        for group in SUITE_GROUPS.values():
            for name in group:
                assert name in names, name

    def test_strict_refuses_to_skip_a_missing_toolchain(self):
        with pytest.raises(UnavailableToolchain, match="cargo"):
            check_tier.all_suites_strict(
                Path("/repo"), available=lambda n: n != "cargo"
            )

    def test_a_bare_run_still_skips_what_the_machine_lacks(self):
        """The local default is untouched. Strictness is opt-in."""
        names = [
            s.name for s in select_suites(Path("/repo"), cargo=False, frontend=False)
        ]
        assert "cargo" not in names

    def test_suite_and_strict_cannot_both_be_given(self):
        """--suite already carries the promise for its group. Two ways to say
        the same thing is one too many."""
        with pytest.raises(SystemExit):
            check_tier.main(["--suite", "backend", "--strict"])


class TestSelectSuites:
    def test_pytest_suite_is_always_selected(self):
        suites = select_suites(Path("/repo"), cargo=False, frontend=False)
        names = [s.name for s in suites]
        assert "pytest" in names

    def test_cargo_suite_selected_only_when_toolchain_present(self):
        without = [
            s.name for s in select_suites(Path("/repo"), cargo=False, frontend=False)
        ]
        assert "cargo" not in without

        with_cargo = [
            s.name for s in select_suites(Path("/repo"), cargo=True, frontend=False)
        ]
        assert "cargo" in with_cargo

    def test_ruff_suites_selected_only_when_the_binary_is_present(self):
        without = [
            s.name
            for s in select_suites(
                Path("/repo"), cargo=False, frontend=False, ruff=False
            )
        ]
        assert "ruff" not in without
        assert "ruff-format" not in without

        with_ruff = [
            s.name for s in select_suites(Path("/repo"), cargo=False, frontend=False)
        ]
        assert "ruff" in with_ruff
        assert "ruff-format" in with_ruff

    def test_lint_and_format_are_separate_suites(self):
        """Run as one, the first failure would hide the second."""
        suites = select_suites(Path("/repo"), cargo=False, frontend=False)
        argvs = {s.name: s.argv for s in suites}
        # argv[0] is a resolved path - the project's copy when there is one.
        assert argvs["ruff"][0].endswith("ruff")
        assert argvs["ruff"][1:] == ["check"]
        assert argvs["ruff-format"][1:] == ["format", "--check"]
        assert argvs["ty"][1:] == ["check"]

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

    def test_frontend_suite_selected_only_when_toolchain_present(self):
        without = [
            s.name for s in select_suites(Path("/repo"), cargo=False, frontend=False)
        ]
        assert "frontend" not in without

        with_fe = [
            s.name for s in select_suites(Path("/repo"), cargo=False, frontend=True)
        ]
        assert "frontend" in with_fe

    def test_frontend_suite_runs_in_the_frontend_directory(self):
        suite = next(
            s
            for s in select_suites(Path("/repo"), cargo=False, frontend=True)
            if s.name == "frontend"
        )
        assert suite.cwd == Path("/repo") / "frontend"

    def test_lint_runs_before_the_suites_it_can_explain(self):
        """A syntax-level break should read as a lint failure, not as a
        pytest collection error further down the run."""
        names = [
            s.name for s in select_suites(Path("/repo"), cargo=True, frontend=True)
        ]
        assert names[:3] == ["ruff", "ruff-format", "ty"]
        # conformance is pytest too, so it needs the tree to parse just as much
        assert names.index("ty") < names.index("conformance")
        assert names.index("conformance") < names.index("pytest")
        assert names.index("pytest") < names.index("cargo")


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
        suite = next(
            s
            for s in select_suites(repo_root, cargo=True, frontend=False)
            if s.name == "cargo"
        )
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

    def test_a_group_names_conformance_or_nothing_runs_it(self):
        """The pytest suite ignores tests/conformance. If no group names the
        conformance suite either, those tests run nowhere and the gate is green
        having skipped a fifth of the suite."""
        grouped = {name for group in SUITE_GROUPS.values() for name in group}
        assert "conformance" in grouped

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


if __name__ == "__main__":
    sys.exit(pytest.main([__file__]))


class TestGroupsAndSelectionAgree:
    """The local gate and the CI stages must cover the same suites.

    A suite added to a group but forgotten in `select_suites` would run in CI
    and silently not run locally - the false-green this module exists to stop.
    """

    def test_every_grouped_suite_is_selectable_locally(self):
        grouped = {name for group in SUITE_GROUPS.values() for name in group}
        selected = {
            s.name for s in select_suites(Path("/repo"), cargo=True, frontend=True)
        }
        assert grouped == selected

    def test_selection_runs_in_group_order(self):
        expected = [name for group in SUITE_GROUPS.values() for name in group]
        actual = [
            s.name for s in select_suites(Path("/repo"), cargo=True, frontend=True)
        ]
        assert actual == expected
