"""sledovane-dokumenty tracking.py — pure/stateful ops: notes, groups, list_all, confirm, remove."""

import pytest

from sledovane_dokumenty_core import state, tracking
from sledovane_dokumenty_core.exceptions import (
    DocumentNotTrackedError,
    GroupNotFoundError,
)


def _seed(codexis_id, **overrides):
    """Create minimal valid state.json."""
    base = {
        "codexisId": codexis_id,
        "name": f"Zákon {codexis_id}",
        "uuid": f"uuid-{codexis_id}",
        "added_on": "2026-03-01T00:00:00Z",
        "last_check_at": "2026-03-01T00:00:00Z",
        "baseline_version_id": "v1",
        "last_known_version_id": "v1",
        "changes": [],
    }
    base.update(overrides)
    state.save_state(codexis_id, base)
    return base


# ── list_all sort ────────────────────────────────────────────────────────────


class TestListAllSort:
    def test_empty(self, dokumenty_app_dir):
        assert tracking.list_all() == []

    def test_unconfirmed_beats_confirmed_beats_idle(self, dokumenty_app_dir):
        _seed("IDLE", changes=[])
        _seed("CONF", changes=[
            {"detected_on": "2026-04-05T00:00:00Z",
             "confirmed_on": "2026-04-06T00:00:00Z"},
        ])
        _seed("PEND", changes=[
            {"detected_on": "2026-04-10T00:00:00Z", "confirmed_on": None},
        ])
        result = tracking.list_all()
        assert [s["codexisId"] for s in result] == ["PEND", "CONF", "IDLE"]

    def test_within_unconfirmed_newest_detection_first(self, dokumenty_app_dir):
        _seed("OLD", changes=[
            {"detected_on": "2026-03-01T00:00:00Z", "confirmed_on": None},
        ])
        _seed("NEW", changes=[
            {"detected_on": "2026-04-15T00:00:00Z", "confirmed_on": None},
        ])
        assert [s["codexisId"] for s in tracking.list_all()] == ["NEW", "OLD"]

    def test_added_on_breaks_tie_among_idle(self, dokumenty_app_dir):
        _seed("A", changes=[], added_on="2026-03-01T00:00:00Z")
        _seed("B", changes=[], added_on="2026-04-01T00:00:00Z")
        assert [s["codexisId"] for s in tracking.list_all()] == ["B", "A"]


# ── show / remove ────────────────────────────────────────────────────────────


class TestShow:
    def test_returns_state(self, dokumenty_app_dir):
        _seed("ZAK1")
        s = tracking.show("ZAK1")
        assert s["codexisId"] == "ZAK1"

    def test_missing_raises(self, dokumenty_app_dir):
        with pytest.raises(DocumentNotTrackedError):
            tracking.show("MISSING")


class TestRemove:
    def test_removes_state_dir(self, dokumenty_app_dir):
        _seed("ZAK1")
        tracking.remove("ZAK1")
        assert state.load_state("ZAK1") is None

    def test_cleans_group_membership(self, dokumenty_app_dir):
        _seed("ZAK1")
        state.save_groups([{"id": "dane", "name": "Daně", "members": ["ZAK1", "ZAK2"]}])
        tracking.remove("ZAK1")
        groups = state.load_groups()
        assert groups[0]["members"] == ["ZAK2"]

    def test_missing_raises(self, dokumenty_app_dir):
        with pytest.raises(DocumentNotTrackedError):
            tracking.remove("MISSING")


# ── confirm ──────────────────────────────────────────────────────────────────


class TestConfirm:
    def test_confirms_single_change_by_index(self, dokumenty_app_dir):
        _seed("Z", changes=[
            {"confirmed_on": None, "old_version_id": "v1", "new_version_id": "v2"},
            {"confirmed_on": None, "old_version_id": "v1", "new_version_id": "v3"},
        ])
        marked = tracking.confirm("Z", change_index=0)
        assert marked == 1
        s = state.load_state("Z")
        assert s["changes"][0]["confirmed_on"] is not None
        assert s["changes"][1]["confirmed_on"] is None

    def test_confirm_by_index_does_not_advance_baseline(self, dokumenty_app_dir):
        _seed("Z", baseline_version_id="v1", last_known_version_id="v2", changes=[
            {"confirmed_on": None, "old_version_id": "v1", "new_version_id": "v2"},
        ])
        tracking.confirm("Z", change_index=0)
        assert state.load_state("Z")["baseline_version_id"] == "v1"

    def test_confirm_all_advances_baseline(self, dokumenty_app_dir):
        _seed("Z", baseline_version_id="v1", last_known_version_id="v5", changes=[
            {"confirmed_on": None, "old_version_id": "v1", "new_version_id": "v5"},
        ])
        marked = tracking.confirm("Z")
        assert marked == 1
        assert state.load_state("Z")["baseline_version_id"] == "v5"

    def test_confirm_already_confirmed_index_returns_zero(self, dokumenty_app_dir):
        _seed("Z", changes=[
            {"confirmed_on": "2026-04-01T00:00:00Z"},
        ])
        assert tracking.confirm("Z", change_index=0) == 0

    def test_confirm_no_unconfirmed_returns_zero(self, dokumenty_app_dir):
        _seed("Z", changes=[
            {"confirmed_on": "2026-04-01T00:00:00Z"},
        ])
        assert tracking.confirm("Z") == 0

    def test_out_of_range_raises(self, dokumenty_app_dir):
        _seed("Z", changes=[{"confirmed_on": None}])
        with pytest.raises(IndexError):
            tracking.confirm("Z", change_index=5)

    def test_missing_doc_raises(self, dokumenty_app_dir):
        with pytest.raises(DocumentNotTrackedError):
            tracking.confirm("MISSING")


# ── notes ────────────────────────────────────────────────────────────────────


class TestNotes:
    def test_add_appends(self, dokumenty_app_dir):
        _seed("Z")
        tracking.note_add("Z", "první")
        tracking.note_add("Z", "druhá")
        assert tracking.note_list("Z") == ["první", "druhá"]

    def test_list_empty(self, dokumenty_app_dir):
        _seed("Z")
        assert tracking.note_list("Z") == []

    def test_remove_at_index(self, dokumenty_app_dir):
        _seed("Z")
        tracking.note_add("Z", "a")
        tracking.note_add("Z", "b")
        tracking.note_add("Z", "c")
        removed = tracking.note_remove("Z", 1)
        assert removed == "b"
        assert tracking.note_list("Z") == ["a", "c"]

    def test_remove_out_of_range_raises(self, dokumenty_app_dir):
        _seed("Z")
        tracking.note_add("Z", "x")
        with pytest.raises(IndexError):
            tracking.note_remove("Z", 5)

    def test_missing_doc_raises_on_all_ops(self, dokumenty_app_dir):
        with pytest.raises(DocumentNotTrackedError):
            tracking.note_add("MISSING", "x")
        with pytest.raises(DocumentNotTrackedError):
            tracking.note_list("MISSING")
        with pytest.raises(DocumentNotTrackedError):
            tracking.note_remove("MISSING", 0)


# ── groups ───────────────────────────────────────────────────────────────────


class TestGroupAdd:
    def test_creates_group_on_first_add(self, dokumenty_app_dir):
        _seed("ZAK1")
        result = tracking.group_add("ZAK1", "Daně")
        assert result == "added"
        groups = state.load_groups()
        assert len(groups) == 1
        assert groups[0]["name"] == "Daně"
        assert groups[0]["id"] == "dane"
        assert "ZAK1" in groups[0]["members"]

    def test_adds_to_existing_group(self, dokumenty_app_dir):
        _seed("ZAK1")
        _seed("ZAK2")
        tracking.group_add("ZAK1", "Daně")
        tracking.group_add("ZAK2", "Daně")
        groups = state.load_groups()
        assert len(groups) == 1
        assert set(groups[0]["members"]) == {"ZAK1", "ZAK2"}

    def test_duplicate_add_returns_already(self, dokumenty_app_dir):
        _seed("ZAK1")
        tracking.group_add("ZAK1", "Daně")
        assert tracking.group_add("ZAK1", "Daně") == "already"

    def test_case_insensitive_group_lookup(self, dokumenty_app_dir):
        _seed("ZAK1")
        _seed("ZAK2")
        tracking.group_add("ZAK1", "Daně")
        # Existing group "Daně" should match even with different casing
        tracking.group_add("ZAK2", "DANĚ")
        assert len(state.load_groups()) == 1

    def test_missing_doc_raises(self, dokumenty_app_dir):
        with pytest.raises(DocumentNotTrackedError):
            tracking.group_add("MISSING", "Daně")


class TestGroupRemove:
    def test_removes_member(self, dokumenty_app_dir):
        _seed("ZAK1")
        tracking.group_add("ZAK1", "Daně")
        tracking.group_remove("ZAK1", "Daně")
        assert state.load_groups()[0]["members"] == []

    def test_missing_group_raises(self, dokumenty_app_dir):
        with pytest.raises(GroupNotFoundError):
            tracking.group_remove("ZAK1", "Nonexistent")

    def test_not_a_member_raises(self, dokumenty_app_dir):
        state.save_groups([{"id": "dane", "name": "Daně", "members": []}])
        with pytest.raises(DocumentNotTrackedError):
            tracking.group_remove("ZAK1", "Daně")


class TestGroupDelete:
    def test_removes_group(self, dokumenty_app_dir):
        state.save_groups([
            {"id": "dane", "name": "Daně", "members": []},
            {"id": "obchod", "name": "Obchod", "members": []},
        ])
        tracking.group_delete("Daně")
        groups = state.load_groups()
        assert len(groups) == 1
        assert groups[0]["name"] == "Obchod"

    def test_missing_raises(self, dokumenty_app_dir):
        with pytest.raises(GroupNotFoundError):
            tracking.group_delete("Nonexistent")


class TestGroupDeleteById:
    def test_removes_by_slug(self, dokumenty_app_dir):
        state.save_groups([
            {"id": "dane", "name": "Daně", "members": []},
            {"id": "obchod", "name": "Obchod", "members": []},
        ])
        tracking.group_delete_by_id("dane")
        groups = state.load_groups()
        assert len(groups) == 1
        assert groups[0]["id"] == "obchod"

    def test_silent_noop_when_missing(self, dokumenty_app_dir):
        state.save_groups([{"id": "a", "name": "A", "members": []}])
        tracking.group_delete_by_id("missing")  # no raise
        assert len(state.load_groups()) == 1


class TestGroupRename:
    def test_updates_name_and_slug(self, dokumenty_app_dir):
        state.save_groups([{"id": "old-name", "name": "Old Name", "members": []}])
        result = tracking.group_rename("old-name", "Nový název")
        assert result is not None
        assert result["name"] == "Nový název"
        assert result["id"] == "novy-nazev"

    def test_returns_none_on_unknown(self, dokumenty_app_dir):
        assert tracking.group_rename("missing", "X") is None


# ── find_by_uuid ─────────────────────────────────────────────────────────────


class TestFindByUuid:
    def test_returns_cid_and_state(self, dokumenty_app_dir):
        _seed("ZAK1", uuid="target-uuid")
        cid, s = tracking.find_by_uuid("target-uuid")
        assert cid == "ZAK1"
        assert s["uuid"] == "target-uuid"

    def test_missing_returns_none_pair(self, dokumenty_app_dir):
        assert tracking.find_by_uuid("missing") == (None, None)

    def test_empty_dir_returns_none_pair(self, dokumenty_app_dir):
        assert tracking.find_by_uuid("anything") == (None, None)
