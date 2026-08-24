"""Unit tests for the Results tracker's record + merge behavior.

Recording lets the marketplace suite run plugins in parallel and still print
each plugin's checks as one contiguous, in-order block instead of interleaved.
"""

from __future__ import annotations

import sys

from tests.harness.output import Results


class TestRecordMode:
    def test_record_buffers_instead_of_printing(self, capsys):
        r = Results(record=True)
        r.ok("installed")
        assert capsys.readouterr().out == ""
        assert any("installed" in line for line in r.lines)

    def test_default_mode_prints_immediately(self, capsys):
        r = Results()
        r.ok("installed")
        assert "installed" in capsys.readouterr().out
        assert r.lines == []

    def test_record_still_counts(self):
        r = Results(record=True)
        r.ok("a")
        r.fail("b")
        r.skip("c")
        assert (r.passed, r.failed, r.skipped) == (1, 1, 1)
        assert r.errors == ["b"]


class TestMerge:
    def test_merge_folds_counts_and_errors(self):
        parent = Results()
        child = Results(record=True)
        child.ok("a")
        child.fail("b broke")
        child.skip("c")
        parent.merge(child)
        assert (parent.passed, parent.failed, parent.skipped) == (1, 1, 1)
        assert parent.errors == ["b broke"]

    def test_merge_replays_child_lines_in_order(self, capsys):
        parent = Results()
        child = Results(record=True)
        child.log("one")
        child.log("two")
        parent.merge(child)
        out = capsys.readouterr().out
        assert out.index("one") < out.index("two")

    def test_merge_into_a_recording_parent_appends_lines(self, capsys):
        parent = Results(record=True)
        child = Results(record=True)
        child.ok("x")
        parent.merge(child)
        assert capsys.readouterr().out == ""
        assert any("x" in line for line in parent.lines)

    def test_two_children_merge_in_the_order_they_are_folded(self, capsys):
        parent = Results()
        first = Results(record=True)
        first.log("first")
        second = Results(record=True)
        second.log("second")
        parent.merge(first)
        parent.merge(second)
        out = capsys.readouterr().out
        assert out.index("first") < out.index("second")


if __name__ == "__main__":
    import pytest

    sys.exit(pytest.main([__file__]))
