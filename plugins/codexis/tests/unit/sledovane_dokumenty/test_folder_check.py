"""Deterministic (AI-free) legislation-change check for watched folders."""

import pytest
from sledovane_dokumenty_core import clients, folder_check, folders, notify


@pytest.fixture
def seeded(watched_root):
    folders.add_watch(watched_root)
    folders.set_references(
        watched_root,
        "./dogs/buy-dog.docx",
        [{"uri": "u", "text": "OZ", "codexisId": "CR26785"}],
    )
    return watched_root


def _mock_versions(monkeypatch, mapping):
    monkeypatch.setattr(clients, "get_versions", lambda cid: mapping[cid])
    monkeypatch.setattr(clients, "get_latest_version_id", lambda vs: vs[0]["versionId"])
    monkeypatch.setattr(
        clients,
        "find_version_info",
        lambda vs, vid: next((v for v in vs if v["versionId"] == vid), None),
    )
    monkeypatch.setattr(clients, "resolve_amendments", lambda vs, vid: [])


def test_seed_sets_baseline_no_change(seeded, monkeypatch):
    _mock_versions(monkeypatch, {"CR26785": [{"versionId": "v1"}]})
    res = folder_check.check_folders(seeded, send_notifications=False)
    assert res["changes_found"] == 0
    tr = folders.read_tracking(seeded)
    assert tr["legislation"]["CR26785"]["baselineVersionId"] == "v1"


def test_detects_version_change_and_is_idempotent(seeded, monkeypatch):
    mapping = {"CR26785": [{"versionId": "v1"}]}
    _mock_versions(monkeypatch, mapping)
    folder_check.check_folders(seeded, send_notifications=False)  # seed at v1

    mapping["CR26785"] = [
        {"versionId": "v2", "validFrom": "2026-07-01"},
        {"versionId": "v1"},
    ]
    res = folder_check.check_folders(seeded, send_notifications=False)
    assert res["changes_found"] == 1
    change = res["results"][0]["changes"][0]
    assert change["old_version_id"] == "v1" and change["new_version_id"] == "v2"
    assert change["effective_on"] == "2026-07-01"
    assert change["compare_url"].startswith("https://next.codexis.cz/porovnat")

    # second run records nothing new
    assert (
        folder_check.check_folders(seeded, send_notifications=False)["changes_found"]
        == 0
    )


def test_confirm_advances_baseline(seeded, monkeypatch):
    mapping = {"CR26785": [{"versionId": "v1"}]}
    _mock_versions(monkeypatch, mapping)
    folder_check.check_folders(seeded, send_notifications=False)
    mapping["CR26785"] = [{"versionId": "v2"}, {"versionId": "v1"}]
    folder_check.check_folders(seeded, send_notifications=False)

    marked = folder_check.confirm_folder(seeded)
    assert marked == 1
    tr = folders.read_tracking(seeded)
    assert tr["legislation"]["CR26785"]["baselineVersionId"] == "v2"
    assert all(c["confirmed_on"] for c in tr["legislation"]["CR26785"]["changes"])


def test_unresolved_references_are_not_tracked(watched_root, monkeypatch):
    folders.add_watch(watched_root)
    folders.set_references(
        watched_root, "./readme.txt", [{"uri": "https://eur-lex", "text": "GDPR"}]
    )
    _mock_versions(monkeypatch, {})
    res = folder_check.check_folders(watched_root, send_notifications=False)
    assert res["changes_found"] == 0
    assert folders.read_tracking(watched_root)["legislation"] == {}


def test_change_triggers_notifications(seeded, monkeypatch):
    mapping = {"CR26785": [{"versionId": "v1"}]}
    _mock_versions(monkeypatch, mapping)
    folder_check.check_folders(seeded, send_notifications=False)
    mapping["CR26785"] = [{"versionId": "v2"}, {"versionId": "v1"}]

    captured = {}

    def fake_notify_changes(results):
        captured["results"] = results
        # The real shape, because check_folders now reads `inApp` out of it.
        return {"email": False, "inApp": True, "changes": 1}

    monkeypatch.setattr(notify, "notify_changes", fake_notify_changes)
    folder_check.check_folders(seeded, send_notifications=True)
    assert "results" in captured
    assert captured["results"][0]["changes"][0]["new_version_id"] == "v2"


# ── a refused bell is its own flag ───────────────────────────────────────────


def _changed_once(monkeypatch, seeded):
    """Get `seeded` into a state where the next check finds one change."""
    mapping = {"CR26785": [{"versionId": "v1"}]}
    _mock_versions(monkeypatch, mapping)
    folder_check.check_folders(seeded, send_notifications=False)
    mapping["CR26785"] = [{"versionId": "v2"}, {"versionId": "v1"}]


def test_notify_failed_is_set_when_the_daemon_refuses_the_bell(seeded, monkeypatch):
    _changed_once(monkeypatch, seeded)
    monkeypatch.setattr(
        notify,
        "notify_changes",
        lambda results: {"email": False, "inApp": False, "changes": 1},
    )

    result = folder_check.check_folders(seeded, send_notifications=True)

    assert result["notify_failed"] is True
    # Its own flag, not another entry in `errors`: the caller ends the run
    # non-zero for this and not for the unrelated problems the check collects.
    assert result["errors"] == []


def test_notify_failed_is_clear_when_no_notification_was_attempted(seeded, monkeypatch):
    _changed_once(monkeypatch, seeded)
    monkeypatch.setattr(
        notify,
        "notify_changes",
        lambda results: {"email": False, "inApp": None, "changes": 1},
    )

    result = folder_check.check_folders(seeded, send_notifications=True)

    # None means the channel is off or there was nothing to send. Neither is a
    # failure, and treating them as one would paint every quiet run red.
    assert result["notify_failed"] is False
