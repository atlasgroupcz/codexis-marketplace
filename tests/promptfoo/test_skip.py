"""Unit tests for run.py's skip filter: a config with a truthy top-level `skip:`
is excluded from the default full run, with its reason surfaced.

No network, no daemon. Run with:
    python3 -m pytest tests/promptfoo/test_skip.py
"""

from __future__ import annotations

from pathlib import Path

from tests.promptfoo import run


class TestConfigSkipReason:
    def test_no_skip_key_runs(self):
        assert run.config_skip_reason({"description": "x"}) is None

    def test_skip_reason_is_returned(self):
        assert run.config_skip_reason({"skip": "needs a CI key"}) == "needs a CI key"

    def test_falsy_skip_is_not_a_skip(self):
        # Empty string / False must not skip - only a real reason does.
        assert run.config_skip_reason({"skip": ""}) is None
        assert run.config_skip_reason({"skip": False}) is None


class TestPartitionSkipped:
    def _write(self, dir_: Path, name: str, body: str) -> Path:
        p = dir_ / name
        p.write_text(body)
        return p

    def test_splits_runnable_from_skipped(self, tmp_path):
        runnable_cfg = self._write(tmp_path, "runs.config.yaml", "description: runs\n")
        skipped_cfg = self._write(
            tmp_path,
            "blocked.config.yaml",
            "skip: waiting for a CI key\ndescription: blocked\n",
        )
        runnable, skipped = run.partition_skipped([runnable_cfg, skipped_cfg])
        assert runnable == [runnable_cfg]
        assert skipped == [("blocked.config.yaml", "waiting for a CI key")]

    def test_all_runnable_when_none_skipped(self, tmp_path):
        a = self._write(tmp_path, "a.config.yaml", "description: a\n")
        b = self._write(tmp_path, "b.config.yaml", "description: b\n")
        runnable, skipped = run.partition_skipped([a, b])
        assert runnable == [a, b]
        assert skipped == []
