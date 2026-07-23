"""Unit tests for the serial/parallel plugin runner.

The marketplace suite installs/uninstalls ~23 plugins, one daemon round-trip at
a time. `run_plugins` can fan that out across worker threads; these tests pin
the two invariants that make a parallel run safe to trust: every plugin runs,
and the merged output reads in manifest order regardless of who finishes first.
"""

from __future__ import annotations

import sys
import threading
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _output import Results
from _plugin_runner import run_plugins


class TestSerial:
    def test_workers_one_runs_every_plugin_once_in_order(self):
        plugins = [{"name": n} for n in ("a", "b", "c")]
        seen = []
        run_plugins(plugins, lambda p, res: seen.append(p["name"]), Results(), workers=1)
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


if __name__ == "__main__":
    import pytest

    sys.exit(pytest.main([__file__]))
