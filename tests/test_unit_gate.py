"""Unit tests for the pre-daemon unit-test gate."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _unit_gate import (
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

    def test_backend_group_covers_plugin_cores_and_binaries(self):
        assert SUITE_GROUPS["backend"] == ("pytest", "cargo")

    def test_frontend_group_covers_the_ui_suite(self):
        assert SUITE_GROUPS["frontend"] == ("frontend",)


class TestSuitesForGroup:
    """Selection only — toolchain presence is stubbed so these stay hermetic."""

    ALL_PRESENT = staticmethod(lambda _name: True)

    def test_backend_selects_pytest_and_cargo_only(self):
        suites = suites_for_group(Path("/repo"), "backend", available=self.ALL_PRESENT)
        assert [s.name for s in suites] == ["pytest", "cargo"]

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
        toolchain the environment lacks — that is the false-green the gate
        exists to prevent. Auto-detection may skip; an explicit request may not.
        """
        with pytest.raises(UnavailableToolchain, match="cargo"):
            suites_for_group(Path("/repo"), "backend", available=lambda name: name != "cargo")


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
        without = [s.name for s in select_suites(Path("/repo"), cargo=False, frontend=False)]
        assert "cargo" not in without

        with_cargo = [s.name for s in select_suites(Path("/repo"), cargo=True, frontend=False)]
        assert "cargo" in with_cargo

    def test_frontend_suite_selected_only_when_toolchain_present(self):
        without = [s.name for s in select_suites(Path("/repo"), cargo=False, frontend=False)]
        assert "frontend" not in without

        with_fe = [s.name for s in select_suites(Path("/repo"), cargo=False, frontend=True)]
        assert "frontend" in with_fe

    def test_frontend_suite_runs_in_the_frontend_directory(self):
        suite = next(
            s for s in select_suites(Path("/repo"), cargo=False, frontend=True)
            if s.name == "frontend"
        )
        assert suite.cwd == Path("/repo") / "frontend"

    def test_pytest_runs_first(self):
        names = [s.name for s in select_suites(Path("/repo"), cargo=True, frontend=True)]
        assert names[0] == "pytest"
        assert names.index("pytest") < names.index("cargo")


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
