"""sledovane-dokumenty tracking.check() — weekly cron orchestration.

Covers both phases:
  1. Version detection: new versions → changes with diff/compare_url/amendments
  2. Related change detection: set-diff per relation type

We mock at the level of clients.* high-level helpers (get_versions, get_meta,
find_version_info, resolve_amendments) and diff.compute_version_diff /
related.detect_changes — the pure transformations they wrap are covered in
test_diff_*.py and test_related.py.
"""

import pytest

from sledovane_dokumenty_core import clients, diff, related, state, tracking
from sledovane_dokumenty_core.exceptions import DocumentNotTrackedError


@pytest.fixture
def mock_all(monkeypatch):
    """Mock clients.* + diff.compute_version_diff + related.detect_changes + summarize_pending."""
    config = {
        "versions_by_id": {},           # {cid: [{"versionId": str, "validFrom": str, ...}]}
        "raise_versions_for": set(),    # set of cids that should make get_versions raise
        "amendments_by_vid": {},        # {version_id: [amendment dicts]}
        "diff_by_pair": {},             # {(baseline_vid, latest_vid): diff_result}
        "related_changes": {},          # {(cid, rtype): change_dict or None}
        "raise_related_for": set(),     # set of (cid, rtype) that raise
    }

    def _get_versions(cid):
        if cid in config["raise_versions_for"]:
            raise clients.CdxClientError(f"network fail for {cid}")
        return config["versions_by_id"].get(cid, [])

    def _find_version_info(versions, vid):
        for v in versions:
            if v.get("versionId") == vid:
                return v
        return None

    def _resolve_amendments(versions, vid):
        return config["amendments_by_vid"].get(vid, [])

    def _compute_version_diff(baseline_vid, latest_vid, parts, printer=None):
        return config["diff_by_pair"].get((baseline_vid, latest_vid), [])

    def _detect_changes(cid, rtype, printer=None):
        key = (cid, rtype)
        if key in config["raise_related_for"]:
            raise clients.CdxClientError(f"related api fail for {rtype}")
        return config["related_changes"].get(key)

    monkeypatch.setattr(clients, "get_versions", _get_versions)
    monkeypatch.setattr(clients, "find_version_info", _find_version_info)
    monkeypatch.setattr(clients, "resolve_amendments", _resolve_amendments)
    monkeypatch.setattr(diff, "compute_version_diff", _compute_version_diff)
    monkeypatch.setattr(related, "detect_changes", _detect_changes)
    # Neutralize summarize_pending (tested-orthogonal LLM flow)
    monkeypatch.setattr(tracking, "summarize_pending", lambda printer=None: None)
    return config


def _seed(codexis_id, baseline_vid="v1", **overrides):
    base = {
        "codexisId": codexis_id,
        "name": f"Zákon {codexis_id}",
        "parts": [],
        "added_on": "2026-03-01T00:00:00Z",
        "baseline_version_id": baseline_vid,
        "last_known_version_id": baseline_vid,
        "last_check_at": "2026-03-01T00:00:00Z",
        "changes": [],
    }
    base.update(overrides)
    state.save_state(codexis_id, base)
    return base


def _versions(*vids):
    return [{"versionId": v, "validFrom": f"2026-0{i+1}-01"} for i, v in enumerate(vids)]


# ── empty / no-change cases ──────────────────────────────────────────────────


class TestCheckNoChanges:
    def test_empty_tracked_list(self, dokumenty_app_dir, mock_all):
        result = tracking.check()
        assert result == {"checked": 0, "changes_found": 0, "errors": []}

    def test_no_change_when_baseline_equals_latest(self, dokumenty_app_dir, mock_all):
        _seed("ZAK1", baseline_vid="v1")
        mock_all["versions_by_id"]["ZAK1"] = _versions("v1")
        result = tracking.check()
        assert result["checked"] == 1
        assert result["changes_found"] == 0
        assert result["errors"] == []
        # last_check_at and last_known_version_id should be updated
        s = state.load_state("ZAK1")
        assert s["last_known_version_id"] == "v1"
        assert s["last_check_at"] != "2026-03-01T00:00:00Z"
        assert s["changes"] == []


# ── new version detection ───────────────────────────────────────────────────


class TestCheckDetectsNewVersion:
    def test_single_doc_new_version(self, dokumenty_app_dir, mock_all):
        _seed("ZAK1", baseline_vid="v1")
        mock_all["versions_by_id"]["ZAK1"] = _versions("v2")
        mock_all["amendments_by_vid"]["v2"] = [{"id": "a1", "name": "Novela 2026"}]
        mock_all["diff_by_pair"][("v1", "v2")] = [{"part": "full", "diff": "diff text"}]

        result = tracking.check()
        assert result["checked"] == 1
        assert result["changes_found"] == 1
        assert result["errors"] == []

        s = state.load_state("ZAK1")
        assert len(s["changes"]) == 1
        change = s["changes"][0]
        assert change["old_version_id"] == "v1"
        assert change["new_version_id"] == "v2"
        assert change["amendments"] == [{"id": "a1", "name": "Novela 2026"}]
        assert change["effective_on"] == "2026-01-01"
        assert change["diffs"] == [{"part": "full", "diff": "diff text"}]
        assert change["confirmed_on"] is None
        assert change["summary_pending"] is True
        assert "compare_url" in change

    def test_last_known_version_id_advances(self, dokumenty_app_dir, mock_all):
        _seed("ZAK1", baseline_vid="v1")
        mock_all["versions_by_id"]["ZAK1"] = _versions("v2")
        tracking.check()
        assert state.load_state("ZAK1")["last_known_version_id"] == "v2"

    def test_baseline_does_not_advance_on_detection(self, dokumenty_app_dir, mock_all):
        # Baseline advances only on confirm(), not check()
        _seed("ZAK1", baseline_vid="v1")
        mock_all["versions_by_id"]["ZAK1"] = _versions("v2")
        tracking.check()
        assert state.load_state("ZAK1")["baseline_version_id"] == "v1"

    def test_duplicate_detection_is_idempotent(self, dokumenty_app_dir, mock_all):
        _seed("ZAK1", baseline_vid="v1")
        mock_all["versions_by_id"]["ZAK1"] = _versions("v2")
        tracking.check()
        # Second run with same versions — no additional change recorded
        tracking.check()
        assert len(state.load_state("ZAK1")["changes"]) == 1


# ── per-doc errors don't affect other docs ──────────────────────────────────


class TestCheckErrorIsolation:
    def test_one_failing_doc_does_not_abort_others(self, dokumenty_app_dir, mock_all):
        _seed("ZAK1", baseline_vid="v1")
        _seed("ZAK2", baseline_vid="v1")
        _seed("ZAK3", baseline_vid="v1")
        mock_all["versions_by_id"]["ZAK1"] = _versions("v1")
        mock_all["raise_versions_for"].add("ZAK2")
        mock_all["versions_by_id"]["ZAK3"] = _versions("v2")

        result = tracking.check()
        assert result["checked"] == 3
        assert len(result["errors"]) == 1
        assert "ZAK2" in result["errors"][0]
        # ZAK3 still detected its change
        assert result["changes_found"] == 1
        assert len(state.load_state("ZAK3")["changes"]) == 1

    def test_empty_versions_list_reported_as_error(
        self, dokumenty_app_dir, mock_all
    ):
        _seed("ZAK1", baseline_vid="v1")
        # versions = [] → get_latest_version_id returns None
        mock_all["versions_by_id"]["ZAK1"] = []
        result = tracking.check()
        assert len(result["errors"]) == 1
        assert "ZAK1" in result["errors"][0] or "Zákon ZAK1" in result["errors"][0]


# ── single doc invocation ────────────────────────────────────────────────────


class TestCheckSingleDoc:
    def test_checks_only_specified_doc(self, dokumenty_app_dir, mock_all):
        _seed("ZAK1", baseline_vid="v1")
        _seed("ZAK2", baseline_vid="v1")
        mock_all["versions_by_id"]["ZAK1"] = _versions("v2")
        mock_all["versions_by_id"]["ZAK2"] = _versions("v2")
        result = tracking.check(codexis_id="ZAK1")
        assert result["checked"] == 1
        # ZAK2 was not touched
        assert state.load_state("ZAK2")["last_known_version_id"] == "v1"

    def test_missing_doc_raises(self, dokumenty_app_dir, mock_all):
        with pytest.raises(DocumentNotTrackedError):
            tracking.check(codexis_id="MISSING")


# ── Phase 1.5: related changes ───────────────────────────────────────────────


class TestCheckRelatedPhase:
    def _seed_with_related(self, codexis_id, types):
        s = _seed(codexis_id)
        s["related_tracking"] = {
            "enabled": True,
            "types": types,
            "last_check_at": "2026-03-01T00:00:00Z",
        }
        state.save_state(codexis_id, s)

    def test_related_change_appended_to_changes(self, dokumenty_app_dir, mock_all):
        self._seed_with_related("ZAK1", ["JUDIKATURA"])
        mock_all["versions_by_id"]["ZAK1"] = _versions("v1")  # no version change
        mock_all["related_changes"][("ZAK1", "JUDIKATURA")] = {
            "change_type": "related_change",
            "relation_type": "JUDIKATURA",
            "relation_type_name": "Judikatura",
            "added_docs": [{"docId": "A", "title": "Rozhodnutí A"}],
            "removed_docs": [],
        }
        result = tracking.check()
        assert result["changes_found"] == 1
        s = state.load_state("ZAK1")
        assert len(s["changes"]) == 1
        assert s["changes"][0]["relation_type"] == "JUDIKATURA"

    def test_related_error_recorded_not_raised(self, dokumenty_app_dir, mock_all):
        self._seed_with_related("ZAK1", ["JUDIKATURA"])
        mock_all["versions_by_id"]["ZAK1"] = _versions("v1")
        mock_all["raise_related_for"].add(("ZAK1", "JUDIKATURA"))
        result = tracking.check()
        assert len(result["errors"]) == 1
        assert "JUDIKATURA" in result["errors"][0]

    def test_related_last_check_at_updated(self, dokumenty_app_dir, mock_all):
        self._seed_with_related("ZAK1", ["JUDIKATURA"])
        mock_all["versions_by_id"]["ZAK1"] = _versions("v1")
        mock_all["related_changes"][("ZAK1", "JUDIKATURA")] = None  # no change
        tracking.check()
        rt = state.load_state("ZAK1")["related_tracking"]
        assert rt["last_check_at"] != "2026-03-01T00:00:00Z"

    def test_disabled_related_skipped(self, dokumenty_app_dir, mock_all):
        s = _seed("ZAK1")
        s["related_tracking"] = {"enabled": False, "types": ["JUDIKATURA"]}
        state.save_state("ZAK1", s)
        mock_all["versions_by_id"]["ZAK1"] = _versions("v1")
        mock_all["related_changes"][("ZAK1", "JUDIKATURA")] = {
            "change_type": "related_change", "added_docs": [], "removed_docs": [],
        }
        result = tracking.check()
        # detect_changes should not be called → no related_change recorded
        assert result["changes_found"] == 0
        assert state.load_state("ZAK1")["changes"] == []

    def test_multiple_related_types_per_doc(self, dokumenty_app_dir, mock_all):
        self._seed_with_related("ZAK1", ["JUDIKATURA", "PROVADECI_PREDPIS"])
        mock_all["versions_by_id"]["ZAK1"] = _versions("v1")
        mock_all["related_changes"][("ZAK1", "JUDIKATURA")] = {
            "change_type": "related_change", "relation_type": "JUDIKATURA",
            "relation_type_name": "Judikatura",
            "added_docs": [{"docId": "J1", "title": "J1"}], "removed_docs": [],
        }
        mock_all["related_changes"][("ZAK1", "PROVADECI_PREDPIS")] = {
            "change_type": "related_change", "relation_type": "PROVADECI_PREDPIS",
            "relation_type_name": "Prováděcí předpis",
            "added_docs": [], "removed_docs": [{"docId": "P1", "title": "P1"}],
        }
        result = tracking.check()
        assert result["changes_found"] == 2
        assert len(state.load_state("ZAK1")["changes"]) == 2
