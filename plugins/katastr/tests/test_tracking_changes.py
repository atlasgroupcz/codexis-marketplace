"""Change detection, payment label, overview entry, and list_all sort order."""

import json
import os

import pytest

from katastr_core.tracking import (
    _detect_changes,
    list_all,
    state_to_overview_entry,
    uhrada_label,
)


def _seed_state(app_dir, cislo_rizeni, **overrides):
    """Write a minimal state.json so list_all/load can see it."""
    base = {
        "uuid": overrides.pop("uuid", f"uuid-{cislo_rizeni}"),
        "cislo_rizeni": cislo_rizeni,
        "typ_rizeni": "V",
        "label": "",
        "stav": "",
        "stav_uhrady": None,
        "datum_prijeti": "",
        "added_on": "2026-04-01T00:00:00Z",
        "last_check_at": "2026-04-01T00:00:00Z",
        "provedene_operace": [],
        "changes": [],
    }
    base.update(overrides)
    dirname = cislo_rizeni.replace("/", "_")
    dpath = os.path.join(str(app_dir), dirname)
    os.makedirs(dpath, exist_ok=True)
    with open(os.path.join(dpath, "state.json"), "w", encoding="utf-8") as f:
        json.dump(base, f)
    return base


# ── _detect_changes ──────────────────────────────────────────────────────────


class TestDetectChanges:
    def _old_state(self, **overrides):
        base = {
            "stav": "V řízení",
            "stav_uhrady": "N",
            "provedene_operace": [
                {"nazev": "Přijetí", "datumProvedeni": "2026-04-01"},
            ],
        }
        base.update(overrides)
        return base

    def _new_data(self, **overrides):
        base = {
            "stav": "V řízení",
            "stavUhrady": "N",
            "provedeneOperace": [
                {"nazev": "Přijetí", "datumProvedeni": "2026-04-01"},
            ],
        }
        base.update(overrides)
        return base

    def test_no_change_returns_none(self):
        assert _detect_changes(self._old_state(), self._new_data()) is None

    def test_stav_change(self):
        change = _detect_changes(
            self._old_state(), self._new_data(stav="Povoleno")
        )
        assert change is not None
        assert change["old_stav"] == "V řízení"
        assert change["new_stav"] == "Povoleno"
        assert change["new_operations"] == []
        assert change["stav_uhrady_changed"] is False
        assert change["confirmed_on"] is None

    def test_new_operation_detected(self):
        new_data = self._new_data(
            provedeneOperace=[
                {"nazev": "Přijetí", "datumProvedeni": "2026-04-01"},
                {"nazev": "Podpis", "datumProvedeni": "2026-04-15"},
            ]
        )
        change = _detect_changes(self._old_state(), new_data)
        assert change is not None
        assert len(change["new_operations"]) == 1
        assert change["new_operations"][0]["nazev"] == "Podpis"

    def test_operation_same_name_different_date_is_new(self):
        new_data = self._new_data(
            provedeneOperace=[
                {"nazev": "Přijetí", "datumProvedeni": "2026-04-01"},
                {"nazev": "Přijetí", "datumProvedeni": "2026-04-20"},
            ]
        )
        change = _detect_changes(self._old_state(), new_data)
        assert change is not None
        assert len(change["new_operations"]) == 1
        assert change["new_operations"][0]["datumProvedeni"] == "2026-04-20"

    def test_uhrada_change(self):
        change = _detect_changes(
            self._old_state(), self._new_data(stavUhrady="U")
        )
        assert change is not None
        assert change["stav_uhrady_changed"] is True
        assert change["old_stav_uhrady"] == "N"
        assert change["new_stav_uhrady"] == "U"

    def test_combined_stav_and_new_op_and_uhrada(self):
        new_data = self._new_data(
            stav="Povoleno",
            stavUhrady="U",
            provedeneOperace=[
                {"nazev": "Přijetí", "datumProvedeni": "2026-04-01"},
                {"nazev": "Povolení vkladu", "datumProvedeni": "2026-04-20"},
            ],
        )
        change = _detect_changes(self._old_state(), new_data)
        assert change is not None
        assert change["old_stav"] == "V řízení"
        assert change["new_stav"] == "Povoleno"
        assert len(change["new_operations"]) == 1
        assert change["stav_uhrady_changed"] is True

    def test_uhrada_none_to_value_is_change(self):
        old = self._old_state(stav_uhrady=None)
        new = self._new_data(stavUhrady="U")
        change = _detect_changes(old, new)
        assert change is not None
        assert change["stav_uhrady_changed"] is True


# ── uhrada_label ─────────────────────────────────────────────────────────────


class TestUhradaLabel:
    def test_uhrazeno(self):
        assert uhrada_label("U") == "Uhrazeno"

    def test_neuhrazeno(self):
        assert uhrada_label("N") == "Neuhrazeno"

    def test_osvobozeno(self):
        assert uhrada_label("O") == "Osvobozeno"

    def test_none(self):
        assert uhrada_label(None) is None

    def test_unknown_code(self):
        assert uhrada_label("X") is None


# ── state_to_overview_entry ──────────────────────────────────────────────────


class TestOverviewEntry:
    def test_minimal_state(self):
        state = {
            "uuid": "abc",
            "cislo_rizeni": "V-1/2026-701",
            "typ_rizeni": "V",
        }
        entry = state_to_overview_entry(state)
        assert entry["uuid"] == "abc"
        assert entry["operace_count"] == 0
        assert entry["changes_count"] == 0
        assert entry["unconfirmed_count"] == 0
        assert entry["last_op_date"] is None
        assert entry["stav_uhrady_label"] is None

    def test_picks_latest_op_date(self):
        state = {
            "provedene_operace": [
                {"nazev": "A", "datumProvedeni": "2026-02-01"},
                {"nazev": "B", "datumProvedeni": "2026-04-01"},
                {"nazev": "C", "datumProvedeni": "2026-03-01"},
            ],
        }
        entry = state_to_overview_entry(state)
        assert entry["last_op_date"] == "2026-04-01"
        assert entry["operace_count"] == 3

    def test_counts_unconfirmed(self):
        state = {
            "changes": [
                {"confirmed_on": None},
                {"confirmed_on": "2026-04-10T00:00:00Z"},
                {"confirmed_on": None},
            ],
        }
        entry = state_to_overview_entry(state)
        assert entry["changes_count"] == 3
        assert entry["unconfirmed_count"] == 2

    def test_maps_uhrada_label(self):
        entry = state_to_overview_entry({"stav_uhrady": "U"})
        assert entry["stav_uhrady_label"] == "Uhrazeno"

    def test_ops_stripped_to_name_and_date(self):
        state = {
            "provedene_operace": [
                {"nazev": "A", "datumProvedeni": "2026-04-01", "extra": "junk"},
            ],
        }
        entry = state_to_overview_entry(state)
        assert entry["provedene_operace"] == [
            {"nazev": "A", "datumProvedeni": "2026-04-01"},
        ]


# ── list_all sort order ──────────────────────────────────────────────────────


class TestListAllSort:
    def test_empty_when_dir_missing(self, app_dir):
        # app_dir fixture creates dir; remove it to simulate pristine state
        os.rmdir(str(app_dir))
        assert list_all() == []

    def test_unconfirmed_beats_confirmed_beats_idle(self, app_dir):
        _seed_state(
            app_dir, "V-1/2026-701",
            changes=[],  # idle
        )
        _seed_state(
            app_dir, "V-2/2026-701",
            changes=[{"detected_on": "2026-04-10T00:00:00Z",
                      "confirmed_on": "2026-04-11T00:00:00Z"}],
        )
        _seed_state(
            app_dir, "V-3/2026-701",
            changes=[{"detected_on": "2026-04-15T00:00:00Z",
                      "confirmed_on": None}],
        )

        result = list_all()
        order = [r["cislo_rizeni"] for r in result]
        assert order == ["V-3/2026-701", "V-2/2026-701", "V-1/2026-701"]

    def test_within_unconfirmed_newest_first(self, app_dir):
        _seed_state(
            app_dir, "V-1/2026-701",
            changes=[{"detected_on": "2026-04-05T00:00:00Z",
                      "confirmed_on": None}],
        )
        _seed_state(
            app_dir, "V-2/2026-701",
            changes=[{"detected_on": "2026-04-18T00:00:00Z",
                      "confirmed_on": None}],
        )
        order = [r["cislo_rizeni"] for r in list_all()]
        assert order == ["V-2/2026-701", "V-1/2026-701"]

    def test_added_on_breaks_tie_among_idle(self, app_dir):
        _seed_state(
            app_dir, "V-1/2026-701",
            changes=[], added_on="2026-03-01T00:00:00Z",
        )
        _seed_state(
            app_dir, "V-2/2026-701",
            changes=[], added_on="2026-04-15T00:00:00Z",
        )
        order = [r["cislo_rizeni"] for r in list_all()]
        assert order == ["V-2/2026-701", "V-1/2026-701"]

    def test_ignores_broken_state_files(self, app_dir):
        _seed_state(app_dir, "V-1/2026-701")
        # Create a directory with corrupted state.json
        bad_dir = os.path.join(str(app_dir), "V-9_2026-701")
        os.makedirs(bad_dir)
        with open(os.path.join(bad_dir, "state.json"), "w") as f:
            f.write("{not valid json")
        result = list_all()
        assert len(result) == 1
        assert result[0]["cislo_rizeni"] == "V-1/2026-701"
