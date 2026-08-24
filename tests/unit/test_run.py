"""Unit tests for the tests/run.py dispatcher."""

from __future__ import annotations

import sys

import pytest

from tests import run


class TestDispatchTable:
    def test_every_tier_has_a_command(self):
        assert set(run.COMMANDS) == {"check", "test", "contract", "marketplace", "eval"}

    @pytest.mark.parametrize(
        "tier,script",
        [
            ("check", "tests.tiers.check"),
            ("test", "tests.tiers.test"),
            ("contract", "tests.tiers.contract"),
            ("marketplace", "tests.tiers.marketplace"),
            ("eval", "tests.promptfoo.run"),
        ],
    )
    def test_tier_maps_to_its_script(self, tier, script):
        assert run.COMMANDS[tier] == script


class TestAllOrdering:
    def test_all_runs_check_first(self):
        assert run.ORDER[0] == "check"

    def test_all_covers_every_command(self):
        assert set(run.ORDER) == set(run.COMMANDS)


class TestBuildArgv:
    def test_runs_the_tier_as_a_module_under_this_interpreter(self):
        """`-m`, not a path: a script run by path gets its own directory on
        sys.path and not the repo root, which is what every tier used to work
        around with a bootstrap of its own."""
        argv = run.build_argv("check", ["--suite", "backend"])
        assert argv[0] == sys.executable
        assert argv[1:3] == ["-m", "tests.tiers.check"]
        assert argv[-2:] == ["--suite", "backend"]

    def test_every_command_names_an_importable_module(self):
        import importlib.util

        for name, module in run.COMMANDS.items():
            assert importlib.util.find_spec(module) is not None, name

    def test_static_tiers_get_no_daemon_arguments(self):
        """Their argparse rejects an unknown flag, so handing them --daemon
        would turn a passthrough into a crash."""
        assert not run.DAEMON_TIERS & {"check", "test"}

    def test_no_tier_runs_another_tier(self):
        """The marketplace tier used to run the whole check gate before touching
        the daemon, skippable with --skip-unit-tests. A pipeline that verifies an
        install has no reason to own the frontend toolchain, and one that dropped
        the flag went red on a turbo it could not find."""
        from pathlib import Path

        from tests.tiers import marketplace

        source = Path(marketplace.__file__).read_text()
        assert "run_default_gate" not in source
        assert "skip-unit-tests" not in source

    def test_unknown_command_raises(self):
        with pytest.raises(KeyError):
            run.build_argv("nope", [])


class TestMain:
    def test_unknown_command_returns_2(self):
        assert run.main(["nope"]) == 2

    def test_all_stops_at_first_failure(self, monkeypatch):
        calls: list[str] = []

        def fake_run_one(command: str, passthrough: list[str]) -> int:
            calls.append(command)
            return 0 if len(calls) == 1 else 1

        monkeypatch.setattr(run, "_run_one", fake_run_one)
        assert run.main(["all"]) == 1
        assert calls == run.ORDER[:2]

    def test_all_routes_passthrough_only_to_daemon_tiers(self, monkeypatch):
        # The check tier takes no daemon args (and rejects unknown flags), so `all`
        # must send passthrough only to the daemon-backed tiers, not to check.
        seen: dict[str, list[str]] = {}

        def fake_run_one(command: str, passthrough: list[str]) -> int:
            seen[command] = passthrough
            return 0

        monkeypatch.setattr(run, "_run_one", fake_run_one)
        assert run.main(["all", "--daemon", "http://x", "--token", "t"]) == 0
        assert seen["check"] == []
        for tier in ("contract", "marketplace", "eval"):
            assert seen[tier] == ["--daemon", "http://x", "--token", "t"]
