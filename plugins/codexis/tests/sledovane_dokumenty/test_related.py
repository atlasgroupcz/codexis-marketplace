"""related.py — related-document tracking: baseline, enable/disable, detect_changes."""

import pytest

from sledovane_dokumenty_core import clients, related, state
from sledovane_dokumenty_core.exceptions import DocumentNotTrackedError


@pytest.fixture
def mock_clients(monkeypatch):
    """Mock clients.fetch_all_related_ids + fetch_doc_title with configurable data.

    Returns a dict used to set per-test response; keys:
      - related_ids_by_type: {relation_type: [doc_ids]}
      - titles: {doc_id: title}
      - raise_on_fetch: bool (default False) — make fetch_all_related_ids raise
    """
    config = {
        "related_ids_by_type": {},
        "titles": {},
        "raise_on_fetch": False,
    }

    def _fetch_all_related_ids(codexis_id, rtype):
        if config["raise_on_fetch"]:
            raise clients.CdxClientError("simulated API failure")
        return config["related_ids_by_type"].get(rtype, [])

    def _fetch_doc_title(doc_id):
        return config["titles"].get(doc_id, f"Title for {doc_id}")

    monkeypatch.setattr(clients, "fetch_all_related_ids", _fetch_all_related_ids)
    monkeypatch.setattr(clients, "fetch_doc_title", _fetch_doc_title)
    return config


def _seed(codexis_id):
    state.save_state(codexis_id, {
        "codexisId": codexis_id,
        "name": f"Zákon {codexis_id}",
        "added_on": "2026-03-01T00:00:00Z",
        "changes": [],
    })


# ── get_type_name ────────────────────────────────────────────────────────────


class TestGetTypeName:
    def test_known_type_returns_czech_label(self):
        assert related.get_type_name("JUDIKATURA") == "Judikatura"
        assert related.get_type_name("PROVADECI_PREDPIS") == "Prováděcí předpis"

    def test_unknown_type_returns_type_code(self):
        assert related.get_type_name("UNKNOWN_TYPE") == "UNKNOWN_TYPE"

    def test_counts_data_override_used_first(self):
        counts = [{"type": "X", "typeName": "Custom X name"}]
        assert related.get_type_name("X", counts_data=counts) == "Custom X name"

    def test_counts_data_missing_type_falls_back(self):
        counts = [{"type": "Y", "typeName": "Y name"}]
        assert related.get_type_name("JUDIKATURA", counts_data=counts) == "Judikatura"


# ── capture_baseline ─────────────────────────────────────────────────────────


class TestCaptureBaseline:
    def test_stores_fetched_ids(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["A", "B", "C"]
        count = related.capture_baseline("ZAK1", "JUDIKATURA")
        assert count == 3
        baseline = state.load_related_baseline("ZAK1", "JUDIKATURA")
        assert baseline["doc_ids"] == ["A", "B", "C"]
        assert baseline["total_count"] == 3
        assert baseline["type"] == "JUDIKATURA"
        assert baseline["captured_at"].endswith("Z")

    def test_empty_fetch_stored_as_empty_baseline(
        self, dokumenty_app_dir, mock_clients
    ):
        _seed("ZAK1")
        count = related.capture_baseline("ZAK1", "JUDIKATURA")
        assert count == 0
        assert state.load_related_baseline("ZAK1", "JUDIKATURA")["doc_ids"] == []


# ── enable_tracking ──────────────────────────────────────────────────────────


class TestEnableTracking:
    def test_first_type_enables(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["A", "B"]
        result = related.enable_tracking("ZAK1", "JUDIKATURA")
        assert result["related_tracking"]["enabled"] is True
        assert result["related_tracking"]["types"] == ["JUDIKATURA"]
        assert state.load_related_baseline("ZAK1", "JUDIKATURA") is not None

    def test_adds_second_type(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        related.enable_tracking("ZAK1", "JUDIKATURA")
        related.enable_tracking("ZAK1", "PROVADECI_PREDPIS")
        types = state.load_state("ZAK1")["related_tracking"]["types"]
        assert set(types) == {"JUDIKATURA", "PROVADECI_PREDPIS"}

    def test_duplicate_is_idempotent(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        related.enable_tracking("ZAK1", "JUDIKATURA")
        before = state.load_related_baseline("ZAK1", "JUDIKATURA")
        related.enable_tracking("ZAK1", "JUDIKATURA")
        after = state.load_related_baseline("ZAK1", "JUDIKATURA")
        # Baseline should not be re-captured on duplicate enable
        assert before["captured_at"] == after["captured_at"]
        types = state.load_state("ZAK1")["related_tracking"]["types"]
        assert types == ["JUDIKATURA"]

    def test_missing_doc_raises(self, dokumenty_app_dir, mock_clients):
        with pytest.raises(DocumentNotTrackedError):
            related.enable_tracking("MISSING", "JUDIKATURA")


# ── disable_tracking ─────────────────────────────────────────────────────────


class TestDisableTracking:
    def test_removes_type_and_baseline(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        related.enable_tracking("ZAK1", "JUDIKATURA")
        related.enable_tracking("ZAK1", "PROVADECI_PREDPIS")
        result = related.disable_tracking("ZAK1", "JUDIKATURA")
        assert "JUDIKATURA" not in result["related_tracking"]["types"]
        assert state.load_related_baseline("ZAK1", "JUDIKATURA") is None
        # Other baseline untouched
        assert state.load_related_baseline("ZAK1", "PROVADECI_PREDPIS") is not None

    def test_last_type_sets_enabled_false(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        related.enable_tracking("ZAK1", "JUDIKATURA")
        result = related.disable_tracking("ZAK1", "JUDIKATURA")
        assert result["related_tracking"]["enabled"] is False
        assert result["related_tracking"]["types"] == []

    def test_disable_non_tracked_type_noop(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        result = related.disable_tracking("ZAK1", "JUDIKATURA")
        # State still has no related_tracking, function doesn't raise
        assert result.get("related_tracking") in (None, {}, {"enabled": False, "types": []})

    def test_missing_doc_raises(self, dokumenty_app_dir, mock_clients):
        with pytest.raises(DocumentNotTrackedError):
            related.disable_tracking("MISSING", "JUDIKATURA")


# ── detect_changes ───────────────────────────────────────────────────────────


class TestDetectChanges:
    def test_none_when_no_baseline(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        assert related.detect_changes("ZAK1", "JUDIKATURA") is None

    def test_none_when_ids_unchanged(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["A", "B"]
        related.enable_tracking("ZAK1", "JUDIKATURA")
        # Same IDs on next fetch
        assert related.detect_changes("ZAK1", "JUDIKATURA") is None

    def test_detects_added_ids(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["A", "B"]
        related.enable_tracking("ZAK1", "JUDIKATURA")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["A", "B", "C"]
        change = related.detect_changes("ZAK1", "JUDIKATURA")
        assert change is not None
        assert change["change_type"] == "related_change"
        assert change["relation_type"] == "JUDIKATURA"
        added_doc_ids = [d["docId"] for d in change["added_docs"]]
        assert added_doc_ids == ["C"]
        assert change["removed_docs"] == []

    def test_detects_removed_ids(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["A", "B", "C"]
        related.enable_tracking("ZAK1", "JUDIKATURA")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["A"]
        change = related.detect_changes("ZAK1", "JUDIKATURA")
        assert change is not None
        removed_ids = [d["docId"] for d in change["removed_docs"]]
        assert set(removed_ids) == {"B", "C"}

    def test_caps_at_10_with_overflow_marker(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = []
        related.enable_tracking("ZAK1", "JUDIKATURA")
        # 15 new IDs
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = [f"D{i}" for i in range(15)]
        change = related.detect_changes("ZAK1", "JUDIKATURA")
        # 10 entries + 1 overflow marker
        assert len(change["added_docs"]) == 11
        assert change["added_docs"][-1]["docId"] == "..."
        assert "a dalších 5" in change["added_docs"][-1]["title"]

    def test_updates_baseline_after_detection(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["A"]
        related.enable_tracking("ZAK1", "JUDIKATURA")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["A", "B"]
        related.detect_changes("ZAK1", "JUDIKATURA")
        baseline = state.load_related_baseline("ZAK1", "JUDIKATURA")
        assert set(baseline["doc_ids"]) == {"A", "B"}

    def test_api_failure_returns_none(self, dokumenty_app_dir, mock_clients):
        _seed("ZAK1")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["A"]
        related.enable_tracking("ZAK1", "JUDIKATURA")
        mock_clients["raise_on_fetch"] = True
        assert related.detect_changes("ZAK1", "JUDIKATURA") is None

    def test_description_md_contains_title_and_link(
        self, dokumenty_app_dir, mock_clients
    ):
        _seed("ZAK1")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = []
        related.enable_tracking("ZAK1", "JUDIKATURA")
        mock_clients["related_ids_by_type"]["JUDIKATURA"] = ["NEWDOC"]
        mock_clients["titles"]["NEWDOC"] = "Rozhodnutí 42/2026"
        change = related.detect_changes("ZAK1", "JUDIKATURA")
        md = change["description_md"]
        assert "Rozhodnutí 42/2026" in md
        assert "Přidáno 1" in md
        assert "Judikatura" in md
