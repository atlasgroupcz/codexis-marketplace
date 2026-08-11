"""Unit tests for the tests/run.py dispatcher."""

from __future__ import annotations

import sys

import pytest

from tests import run


class TestDispatchTable:
    def test_every_tier_has_a_command(self):
        assert set(run.COMMANDS) == {"check", "marketplace", "eval"}

    @pytest.mark.parametrize(
        "tier,script",
        [
            ("check", "_unit_gate.py"),
            ("marketplace", "test-marketplace.py"),
            ("eval", "promptfoo/run.py"),
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
    def test_prefixes_python_and_the_script_path(self):
        argv = run.build_argv("check", ["--suite", "backend"])
        assert argv[0] == sys.executable
        assert argv[1].endswith("tests/_unit_gate.py")
        assert argv[-2:] == ["--suite", "backend"]

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
        for tier in ("marketplace", "eval"):
            assert seen[tier] == ["--daemon", "http://x", "--token", "t"]
