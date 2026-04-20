"""sledovane-dokumenty state.py — slugify, find_group, atomic I/O, related baselines."""

import json
import os

import pytest

from sledovane_dokumenty_core import state


class TestSlugify:
    def test_basic_ascii_lowercase(self):
        assert state.slugify("Hello World") == "hello-world"

    def test_strips_diacritics(self):
        assert state.slugify("Zákon o DPH") == "zakon-o-dph"

    def test_collapses_multiple_separators(self):
        assert state.slugify("a -- b __ c") == "a-b-c"

    def test_strips_leading_trailing_dashes(self):
        assert state.slugify("---hello---") == "hello"

    def test_empty_returns_fallback(self):
        assert state.slugify("") == "skupina"

    def test_only_special_chars_returns_fallback(self):
        assert state.slugify("!!! @@@ ###") == "skupina"

    def test_preserves_digits(self):
        assert state.slugify("Sekce 42") == "sekce-42"


class TestFindGroup:
    def test_case_insensitive_match(self):
        groups = [{"name": "Daně", "members": []}]
        idx, g = state.find_group(groups, "daně")
        assert idx == 0
        assert g["name"] == "Daně"

    def test_not_found_returns_negative(self):
        idx, g = state.find_group([{"name": "A"}], "B")
        assert idx == -1
        assert g is None

    def test_empty_list(self):
        idx, g = state.find_group([], "anything")
        assert idx == -1
        assert g is None

    def test_returns_first_match_by_index(self):
        groups = [
            {"name": "First"},
            {"name": "Second"},
            {"name": "Third"},
        ]
        idx, g = state.find_group(groups, "Second")
        assert idx == 1


# ── state I/O round-trip ─────────────────────────────────────────────────────


class TestStateIO:
    def test_save_load_roundtrip(self, dokumenty_app_dir):
        data = {"codexisId": "ZAK1", "name": "Test", "changes": []}
        state.save_state("ZAK1", data)
        loaded = state.load_state("ZAK1")
        assert loaded == data

    def test_load_missing_returns_none(self, dokumenty_app_dir):
        assert state.load_state("NONEXISTENT") is None

    def test_load_corrupt_json_returns_none(self, dokumenty_app_dir):
        target_dir = dokumenty_app_dir / "BROKEN"
        target_dir.mkdir()
        (target_dir / "state.json").write_text("{not valid")
        assert state.load_state("BROKEN") is None

    def test_save_creates_directory(self, dokumenty_app_dir):
        assert not (dokumenty_app_dir / "NEW").exists()
        state.save_state("NEW", {"x": 1})
        assert (dokumenty_app_dir / "NEW" / "state.json").is_file()

    def test_save_writes_pretty_json_with_trailing_newline(self, dokumenty_app_dir):
        state.save_state("PRETTY", {"a": 1, "b": 2})
        content = (dokumenty_app_dir / "PRETTY" / "state.json").read_text()
        assert content.endswith("\n")
        assert "  " in content  # indented

    def test_atomic_write_leaves_no_tmp_files(self, dokumenty_app_dir):
        state.save_state("ATOMIC", {"x": 1})
        files = os.listdir(dokumenty_app_dir / "ATOMIC")
        assert files == ["state.json"]


class TestAllTrackedIds:
    def test_empty_when_dir_missing(self, dokumenty_app_dir):
        # Fixture creates dir — remove it to simulate pristine state
        os.rmdir(str(dokumenty_app_dir))
        assert state.all_tracked_ids() == []

    def test_empty_when_no_states(self, dokumenty_app_dir):
        assert state.all_tracked_ids() == []

    def test_returns_sorted_ids(self, dokumenty_app_dir):
        state.save_state("ZAK3", {"x": 3})
        state.save_state("ZAK1", {"x": 1})
        state.save_state("ZAK2", {"x": 2})
        assert state.all_tracked_ids() == ["ZAK1", "ZAK2", "ZAK3"]

    def test_skips_entries_without_state_json(self, dokumenty_app_dir):
        state.save_state("GOOD", {"x": 1})
        # Create a stray directory without state.json
        (dokumenty_app_dir / "ORPHAN").mkdir()
        assert state.all_tracked_ids() == ["GOOD"]

    def test_skips_corrupt_state(self, dokumenty_app_dir):
        state.save_state("GOOD", {"x": 1})
        bad_dir = dokumenty_app_dir / "BROKEN"
        bad_dir.mkdir()
        (bad_dir / "state.json").write_text("garbage")
        assert state.all_tracked_ids() == ["GOOD"]


# ── groups I/O ───────────────────────────────────────────────────────────────


class TestGroupsIO:
    def test_load_missing_returns_empty(self, dokumenty_app_dir):
        assert state.load_groups() == []

    def test_save_load_roundtrip(self, dokumenty_app_dir):
        groups = [
            {"id": "dane", "name": "Daně", "members": ["ZAK1"]},
            {"id": "obchodni", "name": "Obchodní", "members": []},
        ]
        state.save_groups(groups)
        assert state.load_groups() == groups

    def test_load_non_list_returns_empty(self, dokumenty_app_dir):
        # Write a JSON object instead of list
        dokumenty_app_dir.mkdir(parents=True, exist_ok=True)
        (dokumenty_app_dir / "groups.json").write_text('{"not": "a list"}')
        assert state.load_groups() == []

    def test_load_corrupt_returns_empty(self, dokumenty_app_dir):
        dokumenty_app_dir.mkdir(parents=True, exist_ok=True)
        (dokumenty_app_dir / "groups.json").write_text("garbage {")
        assert state.load_groups() == []


# ── related baseline I/O ─────────────────────────────────────────────────────


class TestRelatedBaseline:
    def test_save_load_roundtrip(self, dokumenty_app_dir):
        state.save_state("ZAK1", {"codexisId": "ZAK1"})
        baseline = {"type": "JUDIKATURA", "doc_ids": ["A", "B"]}
        state.save_related_baseline("ZAK1", "JUDIKATURA", baseline)
        assert state.load_related_baseline("ZAK1", "JUDIKATURA") == baseline

    def test_load_missing_returns_none(self, dokumenty_app_dir):
        state.save_state("ZAK1", {})
        assert state.load_related_baseline("ZAK1", "MISSING") is None

    def test_baseline_path_includes_type(self, dokumenty_app_dir):
        path = state.related_baseline_path("ZAK1", "JUDIKATURA")
        assert path.endswith("ZAK1/related_JUDIKATURA.json")

    def test_delete_specific_type(self, dokumenty_app_dir):
        state.save_state("ZAK1", {})
        state.save_related_baseline("ZAK1", "A", {"type": "A"})
        state.save_related_baseline("ZAK1", "B", {"type": "B"})
        state.delete_related_baselines("ZAK1", types=["A"])
        assert state.load_related_baseline("ZAK1", "A") is None
        assert state.load_related_baseline("ZAK1", "B") is not None

    def test_delete_all_when_types_none(self, dokumenty_app_dir):
        state.save_state("ZAK1", {})
        state.save_related_baseline("ZAK1", "A", {"type": "A"})
        state.save_related_baseline("ZAK1", "B", {"type": "B"})
        state.delete_related_baselines("ZAK1")
        assert state.load_related_baseline("ZAK1", "A") is None
        assert state.load_related_baseline("ZAK1", "B") is None
        # state.json must still exist
        assert state.load_state("ZAK1") is not None

    def test_delete_nonexistent_no_error(self, dokumenty_app_dir):
        state.delete_related_baselines("NONEXISTENT")
