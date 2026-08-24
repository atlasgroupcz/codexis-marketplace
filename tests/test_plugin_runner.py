"""Unit tests for the serial/parallel plugin runner.

The marketplace suite installs/uninstalls ~23 plugins, one daemon round-trip at
a time. `run_plugins` can fan that out across worker threads; these tests pin
the two invariants that make a parallel run safe to trust: every plugin runs,
and the merged output reads in manifest order regardless of who finishes first.
"""

from __future__ import annotations

import email.message
import sys
import threading
import time
import urllib.error

import pytest

from tests._output import Results
from tests._plugin_runner import RunOne, run_plugins


class TestSerial:
    def test_workers_one_runs_every_plugin_once_in_order(self):
        plugins = [{"name": n} for n in ("a", "b", "c")]
        seen = []
        run_plugins(
            plugins, lambda p, res: seen.append(p["name"]), Results(), workers=1
        )
        assert seen == ["a", "b", "c"]

    def test_counts_aggregate_into_the_parent(self):
        plugins = [{"name": "a"}, {"name": "b"}]
        r = Results()
        run_plugins(plugins, lambda p, res: res.ok(p["name"]), r, workers=1)
        assert r.passed == 2

    def test_empty_plugin_list_is_a_no_op(self):
        r = Results()
        run_plugins([], lambda p, res: res.fail("should not run"), r, workers=4)
        assert (r.passed, r.failed) == (0, 0)


class TestParallel:
    def test_every_plugin_runs_under_multiple_workers(self):
        plugins = [{"name": f"p{i}"} for i in range(6)]
        seen = set()
        lock = threading.Lock()

        def run_one(p, res):
            with lock:
                seen.add(p["name"])
            res.ok(p["name"])

        r = Results()
        run_plugins(plugins, run_one, r, workers=4)
        assert seen == {p["name"] for p in plugins}
        assert r.passed == 6

    def test_output_is_manifest_order_not_completion_order(self):
        """pool.map preserves input order; a regression to as-completed merging
        would interleave. Finish in reverse so the difference is observable."""
        plugins = [{"name": f"p{i}"} for i in range(5)]

        def run_one(p, res):
            idx = int(p["name"][1:])
            time.sleep((5 - idx) * 0.02)  # p4 finishes first, p0 last
            res.log(p["name"])

        r = Results(record=True)
        run_plugins(plugins, run_one, r, workers=5)
        assert all(f"p{i}" in r.lines[i] for i in range(5))

    def test_actually_runs_concurrently(self):
        """With enough workers the slowest plugin bounds wall-clock, not the sum.
        Each task blocks on a barrier that only releases once every worker has
        arrived - it can only trip if they run at the same time."""
        plugins = [{"name": f"p{i}"} for i in range(4)]
        barrier = threading.Barrier(len(plugins), timeout=5)

        def run_one(p, res):
            barrier.wait()  # deadlocks/raises BrokenBarrier if run serially
            res.ok(p["name"])

        r = Results()
        run_plugins(plugins, run_one, r, workers=4)
        assert r.passed == 4

    def test_a_failure_in_one_plugin_does_not_stop_the_others(self):
        plugins = [{"name": f"p{i}"} for i in range(4)]

        def run_one(p, res):
            if p["name"] == "p1":
                res.fail(f"{p['name']} boom")
            else:
                res.ok(p["name"])

        r = Results()
        run_plugins(plugins, run_one, r, workers=3)
        assert r.passed == 3
        assert r.failed == 1


class TestCrashIsolation:
    """An exception escaping `run_one` must stay inside that one plugin.

    The suite reaches the daemon through handlers that catch RuntimeError, which
    is what a GraphQL-level refusal arrives as. A transient gateway 5xx arrives
    as urllib's HTTPError instead - an OSError, not a RuntimeError - so it flies
    past every one of them, unwinds out of the loop and kills the process
    mid-suite: the remaining plugins never run and no summary is printed.
    """

    @staticmethod
    def _raises_503_on(name: str) -> RunOne:
        def run_one(plugin: dict, res: Results) -> None:
            if plugin["name"] == name:
                raise urllib.error.HTTPError(
                    "http://daemon/graphql",
                    503,
                    "Service Unavailable",
                    email.message.Message(),
                    None,
                )
            res.ok(plugin["name"])

        return run_one

    def test_serial_run_reports_the_crash_and_keeps_going(self):
        r = Results()
        run_plugins(
            [{"name": n} for n in ("a", "b", "c")],
            self._raises_503_on("a"),
            r,
            workers=1,
        )
        assert (r.passed, r.failed) == (2, 1)
        assert "a" in r.errors[0] and "503" in r.errors[0]

    def test_parallel_run_reports_the_crash_and_keeps_going(self):
        r = Results()
        run_plugins(
            [{"name": f"p{i}"} for i in range(4)],
            self._raises_503_on("p1"),
            r,
            workers=3,
        )
        assert (r.passed, r.failed) == (3, 1)
        assert "p1" in r.errors[0] and "503" in r.errors[0]

    def test_a_plugin_without_a_name_is_reported_by_id(self):
        """`name` is read off the manifest entry; a crash before that is read
        must still say which plugin it belongs to."""
        r = Results()

        def run_one(plugin: dict, res: Results) -> None:
            raise KeyError("name")

        run_plugins([{"id": "pid-7"}], run_one, r, workers=1)
        assert r.failed == 1 and "pid-7" in r.errors[0]

    def test_keyboard_interrupt_still_ends_the_run(self):
        """The guard is a backstop for failing plugins, not a way to eat ctrl-C."""
        seen = []

        def run_one(plugin: dict, res: Results) -> None:
            seen.append(plugin["name"])
            raise KeyboardInterrupt

        with pytest.raises(KeyboardInterrupt):
            run_plugins([{"name": "a"}, {"name": "b"}], run_one, Results(), workers=1)
        assert seen == ["a"]  # stopped there, never reached b


if __name__ == "__main__":
    sys.exit(pytest.main([__file__]))
