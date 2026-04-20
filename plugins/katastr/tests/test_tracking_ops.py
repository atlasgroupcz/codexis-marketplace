"""Stateful katastr ops: add, remove, show, set_label, find_by_uuid, confirm, check_one."""

import os

import pytest

from katastr_core import tracking
from katastr_core.exceptions import (
    KatastrError,
    ProceedingAlreadyTrackedError,
    ProceedingNotFoundError,
    ProceedingNotTrackedError,
)


@pytest.fixture
def mock_cuzk(monkeypatch):
    """Mock fetch_from_cuzk with configurable per-proceeding responses.

    Returns dict. Keys: "data" maps parsed-number tuple → response dict, and
    "raise_exc" allows simulating API errors.
    """
    config = {"data": {}, "raise_exc": None}

    def _fetch(parsed):
        if config["raise_exc"]:
            raise config["raise_exc"]
        key = (
            parsed["typ_rizeni"], parsed["poradove_cislo"],
            parsed["rok"], parsed["kod_pracoviste"],
        )
        return config["data"].get(key)

    monkeypatch.setattr(tracking, "fetch_from_cuzk", _fetch)
    return config


@pytest.fixture
def noop_automation(monkeypatch):
    """Disable ensure_automation — it's non-fatal but noisy in tests."""
    monkeypatch.setattr(tracking, "ensure_automation", lambda: None)


def _cuzk_record(stav="V řízení", stav_uhrady="N", operations=None):
    return {
        "id": "api-id-xyz",
        "stav": stav,
        "stavUhrady": stav_uhrady,
        "datumPrijeti": "2026-04-01",
        "provedeneOperace": operations or [],
    }


# ── add ──────────────────────────────────────────────────────────────────────


class TestAdd:
    def test_basic_add(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 123, 2026, 701)] = _cuzk_record()
        state = tracking.add("V-123/2026-701")
        assert state["cislo_rizeni"] == "V-123/2026-701"
        assert state["api_id"] == "api-id-xyz"
        assert state["stav"] == "V řízení"
        assert state["stav_uhrady"] == "N"
        assert state["label"] == ""
        assert state["changes"] == []
        assert "uuid" in state

    def test_persists_to_disk(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        tracking.add("V-1/2026-701")
        assert (app_dir / "V-1_2026-701" / "state.json").is_file()

    def test_with_label(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        state = tracking.add("V-1/2026-701", label="Smith v. Jones")
        assert state["label"] == "Smith v. Jones"

    def test_lowercase_normalized(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        state = tracking.add("v-1/2026-701")
        assert state["cislo_rizeni"] == "V-1/2026-701"

    def test_duplicate_raises(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        tracking.add("V-1/2026-701")
        with pytest.raises(ProceedingAlreadyTrackedError):
            tracking.add("V-1/2026-701")

    def test_not_found_raises(self, app_dir, mock_cuzk, noop_automation):
        # mock_cuzk["data"] has no entry → fetch returns None
        with pytest.raises(ProceedingNotFoundError):
            tracking.add("V-999/2026-701")

    def test_api_error_propagates(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["raise_exc"] = KatastrError("API down")
        with pytest.raises(KatastrError):
            tracking.add("V-1/2026-701")


# ── show / remove / set_label / find_by_uuid ─────────────────────────────────


class TestShow:
    def test_returns_state(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        tracking.add("V-1/2026-701")
        assert tracking.show("V-1/2026-701")["cislo_rizeni"] == "V-1/2026-701"

    def test_missing_raises(self, app_dir):
        with pytest.raises(ProceedingNotTrackedError):
            tracking.show("V-999/2026-701")


class TestRemove:
    def test_removes_state(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        tracking.add("V-1/2026-701")
        tracking.remove("V-1/2026-701")
        with pytest.raises(ProceedingNotTrackedError):
            tracking.show("V-1/2026-701")

    def test_missing_raises(self, app_dir):
        with pytest.raises(ProceedingNotTrackedError):
            tracking.remove("V-999/2026-701")


class TestSetLabel:
    def test_updates_label(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        tracking.add("V-1/2026-701")
        tracking.set_label("V-1/2026-701", "Nový štítek")
        assert tracking.show("V-1/2026-701")["label"] == "Nový štítek"

    def test_missing_raises(self, app_dir):
        with pytest.raises(ProceedingNotTrackedError):
            tracking.set_label("V-1/2026-701", "x")


class TestFindByUuid:
    def test_returns_matching(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        added = tracking.add("V-1/2026-701")
        assert tracking.find_by_uuid(added["uuid"])["cislo_rizeni"] == "V-1/2026-701"

    def test_missing_returns_none(self, app_dir):
        assert tracking.find_by_uuid("nonexistent-uuid") is None


# ── confirm ──────────────────────────────────────────────────────────────────


class TestConfirm:
    def _seed_with_changes(self, app_dir, mock_cuzk, noop_automation, changes):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        state = tracking.add("V-1/2026-701")
        state["changes"] = changes
        tracking._save_state("V-1/2026-701", state)

    def test_mark_all_unconfirmed(self, app_dir, mock_cuzk, noop_automation):
        self._seed_with_changes(app_dir, mock_cuzk, noop_automation, [
            {"detected_on": "2026-04-01", "confirmed_on": None},
            {"detected_on": "2026-04-05", "confirmed_on": None},
        ])
        marked = tracking.confirm("V-1/2026-701")
        assert marked == 2
        for c in tracking.show("V-1/2026-701")["changes"]:
            assert c["confirmed_on"] is not None

    def test_mark_specific_index(self, app_dir, mock_cuzk, noop_automation):
        self._seed_with_changes(app_dir, mock_cuzk, noop_automation, [
            {"detected_on": "2026-04-01", "confirmed_on": None},
            {"detected_on": "2026-04-05", "confirmed_on": None},
        ])
        marked = tracking.confirm("V-1/2026-701", change_index=1)
        assert marked == 1
        changes = tracking.show("V-1/2026-701")["changes"]
        assert changes[0]["confirmed_on"] is None
        assert changes[1]["confirmed_on"] is not None

    def test_already_confirmed_index_returns_zero(
        self, app_dir, mock_cuzk, noop_automation
    ):
        self._seed_with_changes(app_dir, mock_cuzk, noop_automation, [
            {"confirmed_on": "2026-04-01T00:00:00Z"},
        ])
        assert tracking.confirm("V-1/2026-701", change_index=0) == 0

    def test_no_unconfirmed_returns_zero(self, app_dir, mock_cuzk, noop_automation):
        self._seed_with_changes(app_dir, mock_cuzk, noop_automation, [
            {"confirmed_on": "2026-04-01T00:00:00Z"},
        ])
        assert tracking.confirm("V-1/2026-701") == 0

    def test_out_of_range_raises(self, app_dir, mock_cuzk, noop_automation):
        self._seed_with_changes(app_dir, mock_cuzk, noop_automation, [
            {"confirmed_on": None},
        ])
        with pytest.raises(IndexError):
            tracking.confirm("V-1/2026-701", change_index=99)

    def test_missing_proceeding_raises(self, app_dir):
        with pytest.raises(ProceedingNotTrackedError):
            tracking.confirm("V-999/2026-701")


# ── check_one ────────────────────────────────────────────────────────────────


class TestCheckOne:
    def test_no_change_returns_ok_no_change(self, app_dir, mock_cuzk, noop_automation):
        base = _cuzk_record()
        mock_cuzk["data"][("V", 1, 2026, 701)] = base
        tracking.add("V-1/2026-701")
        result = tracking.check_one("V-1/2026-701")
        assert result["ok"] is True
        assert result["change"] is None
        assert result["error"] is None

    def test_stav_change_detected_and_recorded(
        self, app_dir, mock_cuzk, noop_automation
    ):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record(stav="V řízení")
        tracking.add("V-1/2026-701")
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record(stav="Povoleno")
        result = tracking.check_one("V-1/2026-701")
        assert result["ok"] is True
        assert result["change"] is not None
        assert result["change"]["old_stav"] == "V řízení"
        assert result["change"]["new_stav"] == "Povoleno"
        # Persisted in state.changes
        assert len(tracking.show("V-1/2026-701")["changes"]) == 1

    def test_api_error_returns_ok_false(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        tracking.add("V-1/2026-701")
        mock_cuzk["raise_exc"] = KatastrError("network down")
        result = tracking.check_one("V-1/2026-701")
        assert result["ok"] is False
        assert result["error"] == "network down"
        assert result["change"] is None

    def test_api_not_found_returns_ok_false(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        tracking.add("V-1/2026-701")
        # Remove from mock → next fetch returns None
        mock_cuzk["data"].pop(("V", 1, 2026, 701))
        result = tracking.check_one("V-1/2026-701")
        assert result["ok"] is False
        assert "nebylo nalezeno" in result["error"]

    def test_updates_last_check_at(self, app_dir, mock_cuzk, noop_automation):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        tracking.add("V-1/2026-701")
        before = tracking.show("V-1/2026-701")["last_check_at"]
        tracking.check_one("V-1/2026-701")
        after = tracking.show("V-1/2026-701")["last_check_at"]
        # after must be ≥ before (ISO 8601 strings sort naturally)
        assert after >= before

    def test_missing_proceeding_raises(self, app_dir):
        with pytest.raises(ProceedingNotTrackedError):
            tracking.check_one("V-999/2026-701")


# ── check_all ────────────────────────────────────────────────────────────────


class TestCheckAll:
    def test_empty_returns_empty_list(self, app_dir):
        assert tracking.check_all() == []

    def test_returns_one_result_per_tracked(
        self, app_dir, mock_cuzk, noop_automation
    ):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        mock_cuzk["data"][("V", 2, 2026, 701)] = _cuzk_record()
        tracking.add("V-1/2026-701")
        tracking.add("V-2/2026-701")
        results = tracking.check_all()
        assert len(results) == 2
        assert all(r["ok"] for r in results)

    def test_error_in_one_does_not_stop_others(
        self, app_dir, mock_cuzk, noop_automation
    ):
        mock_cuzk["data"][("V", 1, 2026, 701)] = _cuzk_record()
        mock_cuzk["data"][("V", 2, 2026, 701)] = _cuzk_record()
        tracking.add("V-1/2026-701")
        tracking.add("V-2/2026-701")

        # Remove one from mock so check finds it missing
        mock_cuzk["data"].pop(("V", 1, 2026, 701))
        results = tracking.check_all()
        assert len(results) == 2
        ok_results = [r for r in results if r["ok"]]
        fail_results = [r for r in results if not r["ok"]]
        assert len(ok_results) == 1
        assert len(fail_results) == 1
