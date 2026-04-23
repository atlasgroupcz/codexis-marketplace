"""sledovana-judikatura state.py — I/O, resolve_uuid, report summaries."""

import pytest

from sledovana_judikatura_core import state
from sledovana_judikatura_core.exceptions import AmbiguousTopicPrefixError


class TestTopicIO:
    def test_save_load_roundtrip(self, judikatura_app_dir):
        data = {"uuid": "abc-123", "name": "Test", "areas": []}
        state.save_topic("abc-123", data)
        assert state.load_topic("abc-123") == data

    def test_load_missing_returns_none(self, judikatura_app_dir):
        assert state.load_topic("nonexistent") is None

    def test_load_corrupt_returns_none(self, judikatura_app_dir):
        tdir = judikatura_app_dir / "corrupt"
        tdir.mkdir()
        (tdir / "topic.json").write_text("not json")
        assert state.load_topic("corrupt") is None

    def test_save_writes_indented_json(self, judikatura_app_dir):
        state.save_topic("uid", {"a": 1, "b": 2})
        content = (judikatura_app_dir / "uid" / "topic.json").read_text()
        assert "  " in content
        assert content.endswith("\n")


class TestResolveUuid:
    def test_exact_full_match_returns_uuid(self, judikatura_app_dir):
        full = "abc123-def456-fullformuuid"
        state.save_topic(full, {"uuid": full, "name": "t"})
        assert state.resolve_uuid(full) == full

    def test_unique_prefix_returns_full_uuid(self, judikatura_app_dir):
        state.save_topic("abc123-topic-one", {"name": "one"})
        state.save_topic("xyz789-topic-two", {"name": "two"})
        assert state.resolve_uuid("abc") == "abc123-topic-one"

    def test_ambiguous_prefix_raises(self, judikatura_app_dir):
        state.save_topic("abc-first", {"name": "a"})
        state.save_topic("abc-second", {"name": "b"})
        with pytest.raises(AmbiguousTopicPrefixError):
            state.resolve_uuid("abc")

    def test_missing_returns_none(self, judikatura_app_dir):
        assert state.resolve_uuid("nonexistent") is None

    def test_missing_app_dir_returns_none(self, judikatura_app_dir):
        import os
        os.rmdir(str(judikatura_app_dir))
        assert state.resolve_uuid("anything") is None

    def test_ambiguous_error_message_reports_count(self, judikatura_app_dir):
        state.save_topic("prefix-a", {})
        state.save_topic("prefix-b", {})
        state.save_topic("prefix-c", {})
        with pytest.raises(AmbiguousTopicPrefixError, match="3 shod"):
            state.resolve_uuid("prefix")


class TestAllTopics:
    def test_empty_when_dir_missing(self, judikatura_app_dir):
        import os
        os.rmdir(str(judikatura_app_dir))
        assert state.all_topics() == []

    def test_returns_sorted_uuid_data_pairs(self, judikatura_app_dir):
        state.save_topic("uuid-c", {"name": "C"})
        state.save_topic("uuid-a", {"name": "A"})
        state.save_topic("uuid-b", {"name": "B"})
        result = state.all_topics()
        uuids = [u for u, _ in result]
        assert uuids == ["uuid-a", "uuid-b", "uuid-c"]

    def test_skips_entries_without_topic_json(self, judikatura_app_dir):
        state.save_topic("good", {"name": "G"})
        (judikatura_app_dir / "orphan").mkdir()
        result = state.all_topics()
        assert [u for u, _ in result] == ["good"]


# ── report I/O ───────────────────────────────────────────────────────────────


class TestReportIO:
    def test_save_load_roundtrip(self, judikatura_app_dir):
        state.save_topic("uid", {})
        report = {"report_id": "2026-04-20", "found_count": 3}
        state.save_report("uid", "2026-04-20", report)
        assert state.load_report("uid", "2026-04-20") == report

    def test_load_missing_returns_none(self, judikatura_app_dir):
        assert state.load_report("uid", "missing") is None

    def test_all_reports_sorted_by_id(self, judikatura_app_dir):
        state.save_topic("uid", {})
        state.save_report("uid", "2026-03-01", {"found_count": 1})
        state.save_report("uid", "2026-04-01", {"found_count": 2})
        state.save_report("uid", "2026-02-01", {"found_count": 3})
        reports = state.all_reports("uid")
        assert [r["found_count"] for r in reports] == [3, 1, 2]  # sorted by date ASC

    def test_all_reports_empty_when_dir_missing(self, judikatura_app_dir):
        state.save_topic("uid", {})
        assert state.all_reports("uid") == []

    def test_all_reports_skips_non_json(self, judikatura_app_dir):
        state.save_topic("uid", {})
        state.save_report("uid", "2026-04-20", {"x": 1})
        rdir = judikatura_app_dir / "uid" / "reports"
        (rdir / "readme.txt").write_text("not a report")
        reports = state.all_reports("uid")
        assert len(reports) == 1

    def test_all_reports_skips_corrupt(self, judikatura_app_dir):
        state.save_topic("uid", {})
        state.save_report("uid", "good", {"x": 1})
        rdir = judikatura_app_dir / "uid" / "reports"
        (rdir / "broken.json").write_text("not valid json")
        reports = state.all_reports("uid")
        assert len(reports) == 1


class TestReportSummaries:
    def test_empty_when_no_reports(self, judikatura_app_dir):
        state.save_topic("uid", {})
        assert state.report_summaries("uid") == []

    def test_projects_expected_fields(self, judikatura_app_dir):
        state.save_topic("uid", {})
        state.save_report("uid", "2026-04-20", {
            "checked_at": "2026-04-20T08:00:00Z",
            "period_from": "2026-04-13",
            "period_to": "2026-04-20",
            "found_count": 5,
            "confirmed_on": None,
            "extra_unused_field": "dropped",
        })
        summaries = state.report_summaries("uid")
        assert len(summaries) == 1
        s = summaries[0]
        assert s["report_id"] == "2026-04-20"
        assert s["checked_at"] == "2026-04-20T08:00:00Z"
        assert s["found_count"] == 5
        assert s["confirmed_on"] is None
        assert "extra_unused_field" not in s

    def test_found_count_defaults_to_zero(self, judikatura_app_dir):
        state.save_topic("uid", {})
        state.save_report("uid", "rid", {})
        assert state.report_summaries("uid")[0]["found_count"] == 0

    def test_sorted_by_report_id(self, judikatura_app_dir):
        state.save_topic("uid", {})
        state.save_report("uid", "2026-04-20", {})
        state.save_report("uid", "2026-02-15", {})
        state.save_report("uid", "2026-03-10", {})
        ids = [s["report_id"] for s in state.report_summaries("uid")]
        assert ids == ["2026-02-15", "2026-03-10", "2026-04-20"]


class TestNowUtc:
    def test_format_is_iso_8601_with_z(self):
        result = state.now_utc()
        assert result.endswith("Z")
        assert "T" in result
        assert len(result) == 20  # YYYY-MM-DDTHH:MM:SSZ
