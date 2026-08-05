"""Unit tests for the pre-daemon unit-test gate."""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

import pytest

from tests import _unit_gate
from tests._unit_gate import (
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
            "pytest",
            "cargo",
        )

    def test_frontend_group_covers_the_ui_suite(self):
        assert SUITE_GROUPS["frontend"] == ("frontend",)


class TestSuitesForGroup:
    """Selection only - toolchain presence is stubbed so these stay hermetic."""

    ALL_PRESENT = staticmethod(lambda _name: True)

    def test_backend_selects_lint_pytest_and_cargo_only(self):
        suites = suites_for_group(Path("/repo"), "backend", available=self.ALL_PRESENT)
        assert [s.name for s in suites] == [
            "ruff",
            "ruff-format",
            "ty",
            "pytest",
            "cargo",
        ]

    def test_frontend_selects_the_frontend_suite_only(self):
        suites = suites_for_group(Path("/repo"), "frontend", available=self.ALL_PRESENT)
        assert [s.name for s in suites] == ["frontend"]

    def test_cargo_suite_runs_in_the_shared_rust_directory(self):
        suites = suites_for_group(Path("/repo"), "backend", available=self.ALL_PRESENT)
        cargo = next(s for s in suites if s.name == "cargo")
        assert cargo.cwd == Path("/repo") / "shared" / "rust"

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
        assert _unit_gate._tool(tmp_path, "ruff") == str(venv_ruff)

    def test_a_tool_with_no_project_copy_falls_back_to_path(self, tmp_path):
        assert _unit_gate._tool(tmp_path, "ruff") == shutil.which("ruff")

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
        assert names[:4] == ["ruff", "ruff-format", "ty", "pytest"]
        assert names.index("pytest") < names.index("cargo")


class TestDefaultRunner:
    def test_cargo_runs_once_at_the_workspace_root(self, monkeypatch):
        """shared/rust is one Cargo workspace, so a single `cargo test` covers
        every member. Walking the crates by hand re-resolves the workspace once
        per crate and silently skips any member not on the hardcoded list.
        """
        calls = []

        class _Completed:
            returncode = 0

        monkeypatch.setattr(
            _unit_gate.subprocess,
            "run",
            lambda argv, cwd: calls.append((argv, cwd)) or _Completed(),
        )

        repo_root = Path(__file__).resolve().parent.parent
        suite = next(
            s
            for s in select_suites(repo_root, cargo=True, frontend=False)
            if s.name == "cargo"
        )
        assert _unit_gate._default_runner(suite) == 0
        assert calls == [(suite.argv, repo_root / "shared" / "rust")]


class TestRunGate:
    def test_returns_zero_when_every_suite_passes(self):
        suites = [Suite("a", ["true"], Path(".")), Suite("b", ["true"], Path("."))]
        assert run_gate(suites, runner=lambda s: 0) == 0

    def test_returns_one_when_any_suite_fails(self):
        suites = [Suite("a", ["true"], Path(".")), Suite("b", ["false"], Path("."))]
        assert run_gate(suites, runner=lambda s: 0 if s.name == "a" else 1) == 1

    def test_runs_every_suite_even_after_a_failure(self):
        """A failing pytest run must not hide a broken cargo suite."""
        seen = []

        def runner(suite):
            seen.append(suite.name)
            return 1

        suites = [Suite("a", ["x"], Path(".")), Suite("b", ["y"], Path("."))]
        run_gate(suites, runner=runner)
        assert seen == ["a", "b"]

    def test_empty_suite_list_passes(self):
        assert run_gate([], runner=lambda s: 1) == 0


if __name__ == "__main__":
    sys.exit(pytest.main([__file__]))
