"""sledovane-dokumenty tracking.add() — registration orchestration with mocked clients."""

import pytest

from sledovane_dokumenty_core import clients, state, tracking
from sledovane_dokumenty_core.exceptions import (
    DocumentAlreadyTrackedError,
    DocumentNotFoundError,
)


@pytest.fixture
def mock_clients(monkeypatch):
    """Mock the clients module functions used by add() / check().

    Config:
      - meta_by_id:    {codexis_id: meta_dict or None}
      - versions_by_id: {codexis_id: versions_list or None}
      - raise_meta:    exception to raise from get_meta (instead of returning)
      - raise_versions: exception to raise from get_versions
    """
    config = {
        "meta_by_id": {},
        "versions_by_id": {},
        "raise_meta": None,
        "raise_versions": None,
    }

    def _get_meta(codexis_id):
        if config["raise_meta"]:
            raise config["raise_meta"]
        return config["meta_by_id"].get(codexis_id)

    def _get_versions(codexis_id):
        if config["raise_versions"]:
            raise config["raise_versions"]
        return config["versions_by_id"].get(codexis_id, [])

    monkeypatch.setattr(clients, "get_meta", _get_meta)
    monkeypatch.setattr(clients, "get_versions", _get_versions)
    # Neutralize subprocess side effect inside add()
    monkeypatch.setattr(tracking, "ensure_check_automation", lambda: None)
    return config


def _meta(title):
    """Build a meta dict in the shape get_doc_name() expects."""
    return {"cr": {"main": {"title": title}}}


def _versions(*version_ids):
    return [{"versionId": vid} for vid in version_ids]


# ── happy path ───────────────────────────────────────────────────────────────


class TestAddHappyPath:
    def test_creates_state_with_expected_shape(self, dokumenty_app_dir, mock_clients):
        mock_clients["meta_by_id"]["ZAK1"] = _meta("Zákon č. 89/2012")
        mock_clients["versions_by_id"]["ZAK1"] = _versions("v5", "v4", "v3")
        s = tracking.add("ZAK1")
        assert s["codexisId"] == "ZAK1"
        assert s["name"] == "Zákon č. 89/2012"
        assert s["baseline_version_id"] == "v5"
        assert s["last_known_version_id"] == "v5"
        assert s["parts"] == []
        assert s["changes"] == []
        assert "uuid" in s
        assert s["added_on"].endswith("Z")
        assert s["last_check_at"].endswith("Z")

    def test_persists_to_disk(self, dokumenty_app_dir, mock_clients):
        mock_clients["meta_by_id"]["ZAK1"] = _meta("Test")
        mock_clients["versions_by_id"]["ZAK1"] = _versions("v1")
        tracking.add("ZAK1")
        loaded = state.load_state("ZAK1")
        assert loaded is not None
        assert loaded["name"] == "Test"

    def test_parts_stored(self, dokumenty_app_dir, mock_clients):
        mock_clients["meta_by_id"]["ZAK1"] = _meta("Test")
        mock_clients["versions_by_id"]["ZAK1"] = _versions("v1")
        s = tracking.add("ZAK1", parts=["§1", "§2"])
        assert s["parts"] == ["§1", "§2"]

    def test_baseline_equals_latest_version(self, dokumenty_app_dir, mock_clients):
        # get_latest_version_id returns versions[0].versionId — first in list
        mock_clients["meta_by_id"]["ZAK1"] = _meta("Test")
        mock_clients["versions_by_id"]["ZAK1"] = _versions("newest", "older", "oldest")
        s = tracking.add("ZAK1")
        assert s["baseline_version_id"] == "newest"


# ── error cases ──────────────────────────────────────────────────────────────


class TestAddErrors:
    def test_already_tracked_raises(self, dokumenty_app_dir, mock_clients):
        mock_clients["meta_by_id"]["ZAK1"] = _meta("Test")
        mock_clients["versions_by_id"]["ZAK1"] = _versions("v1")
        tracking.add("ZAK1")
        with pytest.raises(DocumentAlreadyTrackedError):
            tracking.add("ZAK1")

    def test_meta_missing_raises_not_found(self, dokumenty_app_dir, mock_clients):
        # get_meta returns None → get_doc_name returns None → DocumentNotFoundError
        with pytest.raises(DocumentNotFoundError):
            tracking.add("MISSING")

    def test_meta_without_title_raises(self, dokumenty_app_dir, mock_clients):
        mock_clients["meta_by_id"]["ZAK1"] = {"cr": {"main": {}}}  # no title
        with pytest.raises(DocumentNotFoundError):
            tracking.add("ZAK1")

    def test_empty_versions_raises(self, dokumenty_app_dir, mock_clients):
        mock_clients["meta_by_id"]["ZAK1"] = _meta("Test")
        mock_clients["versions_by_id"]["ZAK1"] = []  # no versions
        with pytest.raises(DocumentNotFoundError):
            tracking.add("ZAK1")

    def test_version_without_versionid_raises(self, dokumenty_app_dir, mock_clients):
        mock_clients["meta_by_id"]["ZAK1"] = _meta("Test")
        # versions[0] has no versionId → get_latest_version_id returns None
        mock_clients["versions_by_id"]["ZAK1"] = [{"noId": "X"}]
        with pytest.raises(DocumentNotFoundError):
            tracking.add("ZAK1")

    def test_meta_api_error_propagates(self, dokumenty_app_dir, mock_clients):
        mock_clients["raise_meta"] = clients.CdxClientError("network")
        with pytest.raises(clients.CdxClientError):
            tracking.add("ZAK1")

    def test_versions_api_error_propagates(self, dokumenty_app_dir, mock_clients):
        mock_clients["meta_by_id"]["ZAK1"] = _meta("Test")
        mock_clients["raise_versions"] = clients.CdxClientError("network")
        with pytest.raises(clients.CdxClientError):
            tracking.add("ZAK1")

    def test_failed_add_does_not_persist_state(
        self, dokumenty_app_dir, mock_clients
    ):
        mock_clients["meta_by_id"]["ZAK1"] = _meta("Test")
        mock_clients["versions_by_id"]["ZAK1"] = []  # will trigger NotFound
        with pytest.raises(DocumentNotFoundError):
            tracking.add("ZAK1")
        assert state.load_state("ZAK1") is None


# ── automation side-effect ───────────────────────────────────────────────────


class TestAddTriggersAutomation:
    def test_ensure_check_automation_called(self, dokumenty_app_dir, monkeypatch):
        # Use direct monkeypatch (bypass the fixture's lambda override)
        calls = []
        monkeypatch.setattr(tracking, "ensure_check_automation", lambda: calls.append(1))
        monkeypatch.setattr(clients, "get_meta", lambda cid: _meta("Test"))
        monkeypatch.setattr(clients, "get_versions", lambda cid: _versions("v1"))
        tracking.add("ZAK1")
        assert calls == [1]
