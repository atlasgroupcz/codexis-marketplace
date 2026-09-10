"""Project resolution, file discovery, scan + watched.json for watched projects."""

import os

import pytest
from sledovane_dokumenty_core import _daemon, nodeids, projects
from sledovane_dokumenty_core.projects import WatchedProjectError

from .conftest import OTHER_PROJECT_ID, PROJECT_ID

CHAT_ID = "9a9a9a9a-0000-4000-8000-00000000000a"


def _chat_in_project(monkeypatch, project_id, name="Smlouvy"):
    """The current chat belongs to `project_id`; the daemon answers both queries."""
    monkeypatch.setenv("CODEXIS_PUBLIC_SESSION_ID", CHAT_ID)

    def fake_graphql(query, variables=None, timeout=10):
        if query == projects._PROJECTS_QUERY:
            return {
                "projects": [
                    {"id": nodeids.encode(("Project", project_id)), "name": name}
                ]
            }
        if query == projects._CHAT_PROJECT_QUERY:
            assert variables == {"id": nodeids.encode(("Chat", CHAT_ID))}
            return {
                "node": {
                    "project": {
                        "id": nodeids.encode(("Project", project_id)),
                        "name": name,
                    }
                }
            }
        return {"node": None}

    monkeypatch.setattr(_daemon, "graphql", fake_graphql)


# ── resolution ───────────────────────────────────────────────────────────────


def test_resolve_by_name_is_case_insensitive(daemon_projects):
    assert projects.resolve("smlouvy")["projectId"] == PROJECT_ID


def test_resolve_by_uuid_and_by_node_id(daemon_projects):
    assert projects.resolve(OTHER_PROJECT_ID)["name"] == "Audit"
    node_id = nodeids.encode(("Project", OTHER_PROJECT_ID))
    assert projects.resolve(node_id)["projectId"] == OTHER_PROJECT_ID


def test_resolve_unknown_project_fails(daemon_projects):
    with pytest.raises(WatchedProjectError):
        projects.resolve("Neexistuje")
    with pytest.raises(WatchedProjectError):
        projects.resolve("5b1c1c4e-0000-4000-8000-0000000000ff")


def test_resolve_ambiguous_name_names_the_candidates(monkeypatch, daemon_projects):
    monkeypatch.setattr(
        _daemon,
        "graphql",
        lambda q, v=None, timeout=10: {
            "projects": [
                {"id": nodeids.encode(("Project", PROJECT_ID)), "name": "Smlouvy"},
                {
                    "id": nodeids.encode(("Project", OTHER_PROJECT_ID)),
                    "name": "smlouvy",
                },
            ]
        },
    )
    with pytest.raises(WatchedProjectError) as exc:
        projects.resolve("Smlouvy")
    assert PROJECT_ID in str(exc.value) and OTHER_PROJECT_ID in str(exc.value)


def test_resolve_empty_uses_the_current_chats_project(monkeypatch, daemon_projects):
    _chat_in_project(monkeypatch, OTHER_PROJECT_ID, name="Audit")
    assert projects.resolve("") == {
        "projectId": OTHER_PROJECT_ID,
        "nodeId": nodeids.encode(("Project", OTHER_PROJECT_ID)),
        "name": "Audit",
    }


def test_resolve_empty_outside_a_project_chat_fails(daemon_projects):
    with pytest.raises(WatchedProjectError):
        projects.resolve(None)


def test_resolve_uuid_falls_back_to_the_index_when_the_daemon_is_down(
    dokumenty_app_dir, monkeypatch
):
    monkeypatch.setattr(_daemon, "graphql", lambda q, v=None, timeout=10: None)
    projects.save_index([{"uuid": "u", "projectId": PROJECT_ID, "name": "Smlouvy"}])
    assert projects.resolve(PROJECT_ID)["name"] == "Smlouvy"
    with pytest.raises(WatchedProjectError):
        projects.resolve(OTHER_PROJECT_ID)


# ── where the files are ──────────────────────────────────────────────────────


def test_files_dir_prefers_the_project_home_on_the_raw_vm(
    watched_project, project_files
):
    assert projects.files_dir(watched_project) == project_files


def test_files_dir_uses_the_project_mount_inside_a_project_chat(
    tmp_path, monkeypatch, dokumenty_app_dir
):
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    mounted = tmp_path / "mounted"
    mounted.mkdir()
    monkeypatch.setattr(projects, "MOUNTED_FILES_DIR", str(mounted))
    _chat_in_project(monkeypatch, PROJECT_ID)

    assert projects.files_dir(PROJECT_ID) == str(mounted)
    assert projects.files_dir(OTHER_PROJECT_ID) is None


def test_files_dir_is_none_elsewhere(tmp_path, monkeypatch, daemon_projects):
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    assert projects.files_dir(PROJECT_ID) is None
    with pytest.raises(WatchedProjectError):
        projects.require_files_dir(PROJECT_ID)


def test_harvest_session_is_the_current_chat_when_it_belongs_to_the_project(
    monkeypatch, daemon_projects
):
    _chat_in_project(monkeypatch, PROJECT_ID)
    assert projects.harvest_session(PROJECT_ID) == CHAT_ID


def test_harvest_session_borrows_the_projects_newest_chat(monkeypatch, daemon_projects):
    def fake_graphql(query, variables=None, timeout=10):
        if query == projects._PROJECT_CHATS_QUERY:
            assert variables == {"id": nodeids.encode(("Project", PROJECT_ID))}
            return {"node": {"chats": [{"id": nodeids.encode(("Chat", CHAT_ID))}]}}
        return {"node": None}

    monkeypatch.setattr(_daemon, "graphql", fake_graphql)
    assert projects.harvest_session(PROJECT_ID) == CHAT_ID


def test_harvest_session_is_none_for_a_project_without_chats(daemon_projects):
    assert projects.harvest_session(PROJECT_ID) is None


def test_harvest_path_is_the_in_chat_mount():
    assert (
        projects.harvest_path("./dogs/buy-dog.docx")
        == "/home/codexis/project/files/dogs/buy-dog.docx"
    )


# ── scan + contract ──────────────────────────────────────────────────────────


def test_scan_relative_paths_and_checksums(project_files):
    files = projects.scan_files(project_files)
    assert [f["path"] for f in files] == ["./dogs/buy-dog.docx", "./readme.txt"]
    assert all(len(f["sha256"]) == 64 for f in files)


def test_scan_skips_hidden_entries(project_files):
    os.makedirs(os.path.join(project_files, ".hidden"))
    with open(os.path.join(project_files, ".hidden", "x.txt"), "w") as f:
        f.write("{}")
    paths = [f["path"] for f in projects.scan_files(project_files)]
    assert all(".hidden" not in p for p in paths)


def test_add_watch_writes_contract_in_app_state(watched_project, dokumenty_app_dir):
    res = projects.add_watch("Smlouvy")
    assert res["projectId"] == watched_project
    assert res["files_available"] is True
    assert res["total"] == 2 and res["added"] == 2

    data = projects.read_watched(watched_project)
    assert data["version"] == projects.CONTRACT_VERSION
    assert data["projectId"] == watched_project
    assert {d["path"] for d in data["documents"]} == {
        "./dogs/buy-dog.docx",
        "./readme.txt",
    }
    doc = next(d for d in data["documents"] if d["path"] == "./readme.txt")
    assert doc["extractedAt"] is None and doc["legislation"] == []

    expected = os.path.join(
        str(dokumenty_app_dir), "projects", watched_project, "watched.json"
    )
    assert os.path.isfile(expected)


def test_add_watch_registers_index_and_is_idempotent(watched_project):
    projects.add_watch(watched_project)
    index = projects.load_index()
    assert len(index) == 1
    assert index[0]["projectId"] == watched_project
    assert index[0]["name"] == "Smlouvy"
    projects.add_watch("Smlouvy")
    assert len(projects.load_index()) == 1


def test_add_watch_without_visible_files_registers_an_empty_contract(
    tmp_path, monkeypatch, dokumenty_app_dir, daemon_projects
):
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    res = projects.add_watch("Audit")
    assert res["files_available"] is False and res["total"] == 0
    assert projects.read_watched(OTHER_PROJECT_ID)["documents"] == []
    assert projects.find_in_index(OTHER_PROJECT_ID)["name"] == "Audit"


def test_sync_detects_new_changed_removed(watched_project, project_files):
    projects.add_watch(watched_project)

    with open(os.path.join(project_files, "dogs", "buy-dog.docx"), "w") as f:
        f.write("ZMENA 89/2012 Sb.")
    with open(os.path.join(project_files, "new.txt"), "w") as f:
        f.write("novy")
    os.remove(os.path.join(project_files, "readme.txt"))

    _, summary = projects.sync_watched(watched_project)
    assert summary == {"added": 1, "changed": 1, "removed": 1, "total": 2}


def test_changed_checksum_resets_extraction(watched_project, project_files):
    projects.add_watch(watched_project)
    projects.set_references(
        watched_project, "./readme.txt", [{"uri": "u", "text": "t"}]
    )
    assert "./readme.txt" not in projects.docs_needing_harvest(watched_project)

    with open(os.path.join(project_files, "readme.txt"), "w") as f:
        f.write("zcela jiny obsah")
    projects.sync_watched(watched_project)
    assert "./readme.txt" in projects.docs_needing_harvest(watched_project)


def test_set_references_stamps_timestamps(watched_project):
    projects.add_watch(watched_project)
    doc = projects.set_references(
        watched_project,
        "./dogs/buy-dog.docx",
        [{"uri": "https://x", "text": "Zákon", "codexisId": "CR1"}],
    )
    assert doc["extractedAt"] is not None
    assert doc["legislation"][0]["codexisId"] == "CR1"


def test_unique_legislation_dedupes(watched_project):
    projects.add_watch(watched_project)
    ref = [{"uri": "u", "text": "OZ", "codexisId": "CR26785"}]
    projects.set_references(watched_project, "./dogs/buy-dog.docx", ref)
    projects.set_references(watched_project, "./readme.txt", ref)
    uniq = projects.unique_legislation(watched_project)
    assert len(uniq) == 1 and uniq[0]["codexisId"] == "CR26785"


def test_mark_all_for_reharvest(watched_project):
    projects.add_watch(watched_project)
    projects.set_references(
        watched_project, "./readme.txt", [{"uri": "u", "text": "t"}]
    )
    assert projects.mark_all_for_reharvest(watched_project) == 1
    assert sorted(projects.docs_needing_harvest(watched_project)) == [
        "./dogs/buy-dog.docx",
        "./readme.txt",
    ]


def test_list_watches_counts(watched_project):
    projects.add_watch(watched_project)
    projects.set_references(
        watched_project,
        "./readme.txt",
        [{"uri": "u", "text": "OZ", "codexisId": "CR26785"}],
    )
    [entry] = projects.list_watches()
    assert entry["documents"] == 2
    assert entry["pending_harvest"] == 1
    assert entry["legislation"] == 1
    assert entry["unconfirmed_changes"] == 0


def test_remove_watch_and_purge(watched_project, dokumenty_app_dir):
    projects.add_watch(watched_project)
    assert projects.remove_watch(watched_project) is True
    assert projects.load_index() == []
    assert projects.read_watched(watched_project) is not None
    assert projects.remove_watch(watched_project, purge=True) is False
    assert projects.read_watched(watched_project) is None


def test_available_to_watch_excludes_watched(watched_project):
    projects.add_watch(watched_project)
    assert [p["projectId"] for p in projects.available_to_watch()] == [OTHER_PROJECT_ID]


def test_available_to_watch_needs_the_daemon(monkeypatch, dokumenty_app_dir):
    monkeypatch.setattr(_daemon, "graphql", lambda q, v=None, timeout=10: None)
    with pytest.raises(WatchedProjectError):
        projects.available_to_watch()


# ── node id codec ────────────────────────────────────────────────────────────


def test_nodeids_round_trip_and_daemon_sample():
    encoded = nodeids.encode(("Project", PROJECT_ID))
    assert nodeids.decode(encoded) == [("Project", PROJECT_ID)]
    assert nodeids.root_id(encoded, "Project") == PROJECT_ID
    assert nodeids.root_id(encoded, "Chat") is None
    assert nodeids.decode("D1B1YmxpY0lzb2xhdGlvbgxtYXJrZXRwbGFjZXM") == [
        ("PublicIsolation", "marketplaces")
    ]
    assert nodeids.root_id("not base64!", "Project") is None
