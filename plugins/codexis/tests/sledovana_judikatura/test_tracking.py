"""sledovana-judikatura tracking.py — topic lifecycle, notes, areas, reports."""

import io
import json

import pytest

from sledovana_judikatura_core import state, tracking
from sledovana_judikatura_core.exceptions import (
    AreaAlreadyExistsError,
    AreaIndexError,
    NoteIndexError,
    ReportNotFoundError,
    ReportSourceError,
    TopicNotFoundError,
)


# ── topic lifecycle ──────────────────────────────────────────────────────────


class TestInit:
    def test_creates_topic_with_uuid(self, judikatura_app_dir):
        topic_uuid, data = tracking.init("Náhrada škody")
        assert data["name"] == "Náhrada škody"
        assert data["uuid"] == topic_uuid
        assert data["notes"] == []
        assert data["areas"] == []
        assert data["last_check_at"] is None

    def test_initial_note_stored_as_list(self, judikatura_app_dir):
        _, data = tracking.init("Téma", notes="první poznámka")
        assert data["notes"] == ["první poznámka"]

    def test_persists_to_disk(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("Téma")
        loaded = state.load_topic(topic_uuid)
        assert loaded is not None
        assert loaded["name"] == "Téma"


class TestShow:
    def test_returns_full_uuid_data_summaries(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("Téma")
        full, data, summaries = tracking.show(topic_uuid)
        assert full == topic_uuid
        assert data["name"] == "Téma"
        assert summaries == []

    def test_works_with_prefix(self, judikatura_app_dir):
        full_uuid = "abc123-unique-prefix"
        state.save_topic(full_uuid, {"name": "T", "uuid": full_uuid})
        full, _, _ = tracking.show("abc")
        assert full == full_uuid

    def test_missing_raises(self, judikatura_app_dir):
        with pytest.raises(TopicNotFoundError):
            tracking.show("nonexistent")


class TestDelete:
    def test_removes_from_disk(self, judikatura_app_dir, monkeypatch):
        monkeypatch.setattr("subprocess.run", lambda *a, **k: None)
        topic_uuid, _ = tracking.init("To delete")
        tracking.delete(topic_uuid)
        assert state.load_topic(topic_uuid) is None

    def test_calls_cdxctl_when_automation_id_present(
        self, judikatura_app_dir, monkeypatch
    ):
        calls = []
        monkeypatch.setattr(
            "subprocess.run", lambda *a, **k: calls.append(a[0])
        )
        topic_uuid, _ = tracking.init("With auto")
        tracking.set_automation(topic_uuid, "auto-42")
        tracking.delete(topic_uuid)
        assert any(
            "automation" in str(c) and "auto-42" in str(c) for c in calls
        )

    def test_skips_cdxctl_without_automation_id(
        self, judikatura_app_dir, monkeypatch
    ):
        calls = []
        monkeypatch.setattr(
            "subprocess.run", lambda *a, **k: calls.append(a[0])
        )
        topic_uuid, _ = tracking.init("No auto")
        tracking.delete(topic_uuid)
        assert calls == []

    def test_subprocess_failure_does_not_raise(
        self, judikatura_app_dir, monkeypatch
    ):
        def boom(*a, **k):
            raise RuntimeError("cdxctl not reachable")

        monkeypatch.setattr("subprocess.run", boom)
        topic_uuid, _ = tracking.init("Boom")
        tracking.set_automation(topic_uuid, "auto-x")
        # Must not raise even if cdxctl call fails
        tracking.delete(topic_uuid)
        assert state.load_topic(topic_uuid) is None

    def test_missing_topic_raises(self, judikatura_app_dir):
        with pytest.raises(TopicNotFoundError):
            tracking.delete("nonexistent")


class TestTouch:
    def test_updates_last_check_at(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        _, ts = tracking.touch(topic_uuid)
        assert ts.endswith("Z")
        assert state.load_topic(topic_uuid)["last_check_at"] == ts


class TestSetAutomation:
    def test_stores_id(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        tracking.set_automation(topic_uuid, "auto-99")
        assert state.load_topic(topic_uuid)["automation_id"] == "auto-99"

    def test_overwrites_existing(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        tracking.set_automation(topic_uuid, "first")
        tracking.set_automation(topic_uuid, "second")
        assert state.load_topic(topic_uuid)["automation_id"] == "second"


# ── list_all sort ────────────────────────────────────────────────────────────


class TestListAll:
    def test_empty(self, judikatura_app_dir):
        assert tracking.list_all() == []

    def test_unconfirmed_beats_confirmed_beats_idle(self, judikatura_app_dir):
        # idle topic — no reports
        state.save_topic("idle", {
            "name": "Idle", "uuid": "idle",
            "created_at": "2026-03-01T00:00:00Z",
        })
        # topic with all confirmed reports
        state.save_topic("confirmed-only", {
            "name": "Confirmed", "uuid": "confirmed-only",
            "created_at": "2026-03-15T00:00:00Z",
        })
        state.save_report("confirmed-only", "2026-04-10", {
            "checked_at": "2026-04-10T00:00:00Z",
            "confirmed_on": "2026-04-11T00:00:00Z",
            "found_count": 1,
        })
        # topic with unconfirmed report
        state.save_topic("pending", {
            "name": "Pending", "uuid": "pending",
            "created_at": "2026-03-20T00:00:00Z",
        })
        state.save_report("pending", "2026-04-15", {
            "checked_at": "2026-04-15T00:00:00Z",
            "confirmed_on": None,
            "found_count": 2,
        })

        result = tracking.list_all()
        order = [t["uuid"] for t in result]
        assert order == ["pending", "confirmed-only", "idle"]

    def test_report_summaries_field_stripped(self, judikatura_app_dir):
        tracking.init("T")
        for t in tracking.list_all():
            assert "_report_summaries" not in t

    def test_unconfirmed_count_reported(self, judikatura_app_dir):
        state.save_topic("t", {"name": "T", "uuid": "t"})
        state.save_report("t", "r1", {"confirmed_on": None})
        state.save_report("t", "r2", {"confirmed_on": "2026-04-10T00:00:00Z"})
        state.save_report("t", "r3", {"confirmed_on": None})
        entry = tracking.list_all()[0]
        assert entry["total_reports"] == 3
        assert entry["unconfirmed_reports"] == 2


# ── notes ────────────────────────────────────────────────────────────────────


class TestNotes:
    def test_add_returns_index(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        idx = tracking.note_add(topic_uuid, "první")
        assert idx == 0
        idx2 = tracking.note_add(topic_uuid, "druhá")
        assert idx2 == 1

    def test_list_returns_all(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T", notes="init note")
        tracking.note_add(topic_uuid, "další")
        assert tracking.note_list(topic_uuid) == ["init note", "další"]

    def test_remove_at_index(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        tracking.note_add(topic_uuid, "a")
        tracking.note_add(topic_uuid, "b")
        tracking.note_add(topic_uuid, "c")
        removed = tracking.note_remove(topic_uuid, 1)
        assert removed == "b"
        assert tracking.note_list(topic_uuid) == ["a", "c"]

    def test_remove_out_of_range_raises(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        tracking.note_add(topic_uuid, "a")
        with pytest.raises(NoteIndexError):
            tracking.note_remove(topic_uuid, 5)

    def test_remove_negative_raises(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T", notes="x")
        with pytest.raises(NoteIndexError):
            tracking.note_remove(topic_uuid, -1)


# ── areas ────────────────────────────────────────────────────────────────────


class TestAreas:
    def test_add_returns_index(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        assert tracking.area_add(topic_uuid, "Náhrada škody") == 0
        assert tracking.area_add(topic_uuid, "Ochrana spotřebitele") == 1

    def test_add_duplicate_case_insensitive_raises(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        tracking.area_add(topic_uuid, "Náhrada škody")
        with pytest.raises(AreaAlreadyExistsError):
            tracking.area_add(topic_uuid, "náhrada škody")

    def test_area_has_baseline_none_and_timestamp(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        tracking.area_add(topic_uuid, "A")
        areas = tracking.area_list(topic_uuid)
        assert areas[0]["baseline_summary"] is None
        assert areas[0]["added_at"].endswith("Z")

    def test_remove_by_index(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        tracking.area_add(topic_uuid, "A")
        tracking.area_add(topic_uuid, "B")
        removed = tracking.area_remove(topic_uuid, 0)
        assert removed["name"] == "A"
        assert [a["name"] for a in tracking.area_list(topic_uuid)] == ["B"]

    def test_remove_out_of_range_raises(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        with pytest.raises(AreaIndexError):
            tracking.area_remove(topic_uuid, 0)

    def test_set_baseline_updates_summary_and_ts(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        tracking.area_add(topic_uuid, "A")
        area = tracking.area_set_baseline(topic_uuid, 0, "summary text")
        assert area["baseline_summary"] == "summary text"
        assert area["baseline_set_at"].endswith("Z")

    def test_set_baseline_out_of_range_raises(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        with pytest.raises(AreaIndexError):
            tracking.area_set_baseline(topic_uuid, 3, "x")


# ── reports ──────────────────────────────────────────────────────────────────


class TestSaveReport:
    def test_from_file(self, judikatura_app_dir, tmp_path):
        topic_uuid, _ = tracking.init("T")
        report_path = tmp_path / "report.json"
        report_path.write_text(json.dumps({
            "report_id": "2026-04-20",
            "found_count": 7,
        }))
        rid = tracking.save_report(topic_uuid, file_path=str(report_path))
        assert rid == "2026-04-20"
        loaded = state.load_report(topic_uuid, "2026-04-20")
        assert loaded["found_count"] == 7
        assert loaded["confirmed_on"] is None  # auto-added
        assert "checked_at" in loaded  # auto-added

    def test_from_stream(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        stream = io.StringIO(json.dumps({"report_id": "r1", "found_count": 2}))
        rid = tracking.save_report(topic_uuid, stream=stream)
        assert rid == "r1"

    def test_auto_generates_date_report_id(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        stream = io.StringIO(json.dumps({"found_count": 1}))
        rid = tracking.save_report(topic_uuid, stream=stream)
        # Default: today's UTC date, YYYY-MM-DD
        assert len(rid) == 10 and rid[4] == "-" and rid[7] == "-"

    def test_updates_last_check_at(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        stream = io.StringIO(json.dumps({
            "report_id": "r1",
            "checked_at": "2026-04-20T10:00:00Z",
        }))
        tracking.save_report(topic_uuid, stream=stream)
        data = state.load_topic(topic_uuid)
        assert data["last_check_at"] == "2026-04-20T10:00:00Z"

    def test_missing_file_raises(self, judikatura_app_dir, tmp_path):
        topic_uuid, _ = tracking.init("T")
        with pytest.raises(ReportSourceError):
            tracking.save_report(topic_uuid, file_path=str(tmp_path / "nope.json"))

    def test_invalid_json_raises(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        stream = io.StringIO("not json {")
        with pytest.raises(ReportSourceError):
            tracking.save_report(topic_uuid, stream=stream)


class TestListAndShowReport:
    def test_list_reports(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        state.save_report(topic_uuid, "r1", {"found_count": 1})
        assert len(tracking.list_reports(topic_uuid)) == 1

    def test_show_report(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        state.save_report(topic_uuid, "r1", {"found_count": 5})
        assert tracking.show_report(topic_uuid, "r1")["found_count"] == 5

    def test_show_missing_raises(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        with pytest.raises(ReportNotFoundError):
            tracking.show_report(topic_uuid, "missing")


class TestConfirmReport:
    def test_sets_confirmed_on(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        state.save_report(topic_uuid, "r1", {"confirmed_on": None})
        ts = tracking.confirm_report(topic_uuid, "r1")
        assert ts.endswith("Z")
        assert state.load_report(topic_uuid, "r1")["confirmed_on"] == ts

    def test_missing_raises(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        with pytest.raises(ReportNotFoundError):
            tracking.confirm_report(topic_uuid, "missing")


# ── find_topic (best-effort lookup) ──────────────────────────────────────────


class TestFindTopic:
    def test_returns_data_on_match(self, judikatura_app_dir):
        topic_uuid, _ = tracking.init("T")
        full, data = tracking.find_topic(topic_uuid)
        assert full == topic_uuid
        assert data["name"] == "T"

    def test_returns_none_on_miss(self, judikatura_app_dir):
        assert tracking.find_topic("nonexistent") == (None, None)

    def test_never_raises_on_ambiguous(self, judikatura_app_dir):
        state.save_topic("dup-1", {"name": "A"})
        state.save_topic("dup-2", {"name": "B"})
        # resolve_uuid would raise AmbiguousTopicPrefixError, find_topic swallows
        assert tracking.find_topic("dup") == (None, None)
