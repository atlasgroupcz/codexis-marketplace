"""Project resolution, folder discovery, scan + watched.json for watched folders."""

import os

import pytest
from sledovane_dokumenty_core import _daemon, nodeids, projects, state
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
    projects.save_index(
        [{"uuid": "u", "projectId": PROJECT_ID, "name": "Smlouvy", "path": "."}]
    )
    assert projects.resolve(PROJECT_ID)["name"] == "Smlouvy"
    with pytest.raises(WatchedProjectError):
        projects.resolve(OTHER_PROJECT_ID)


# ── folders inside the project ───────────────────────────────────────────────


def test_normalize_folder_keeps_paths_inside_the_project():
    assert projects.normalize_folder(None) == "."
    assert projects.normalize_folder("/") == "."
    assert projects.normalize_folder("./smlouvy/") == "smlouvy"
    assert projects.normalize_folder("smlouvy/./2026") == "smlouvy/2026"
    assert projects.normalize_folder("/home/codexis/project/files/smlouvy") == "smlouvy"
    for escape in ("..", "../x", "smlouvy/../../x"):
        with pytest.raises(WatchedProjectError):
            projects.normalize_folder(escape)


def test_folder_label_reads_as_a_path():
    assert projects.folder_label(".") == "/"
    assert projects.folder_label("smlouvy/2026") == "/smlouvy/2026"


def test_browse_lists_sub_folders_of_the_project(watched_project):
    root = projects.browse("Smlouvy")
    assert root["name"] == "Smlouvy" and root["path"] == "." and root["parent"] is None
    assert [e["path"] for e in root["entries"]] == ["smlouvy"]

    inner = projects.browse(PROJECT_ID, "smlouvy")
    assert inner["parent"] == "."
    assert [e["path"] for e in inner["entries"]] == ["smlouvy/dogs"]
    assert projects.browse(PROJECT_ID, "smlouvy/dogs")["parent"] == "smlouvy"


def test_browse_rejects_a_missing_folder(watched_project):
    with pytest.raises(WatchedProjectError):
        projects.browse("Smlouvy", "neexistuje")


def test_browse_needs_the_files(tmp_path, monkeypatch, daemon_projects):
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    with pytest.raises(WatchedProjectError):
        projects.browse("Smlouvy")


# ── where the files are ──────────────────────────────────────────────────────


def test_files_dir_prefers_the_project_home_on_the_raw_vm(
    watched_project, project_files
):
    assert projects.files_dir(watched_project) == project_files
    assert projects.folder_dir(watched_project, "smlouvy") == os.path.join(
        project_files, "smlouvy"
    )


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
    assert projects.folder_dir(PROJECT_ID, "smlouvy") is None
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
        projects.harvest_path(".", "./a.docx") == "/home/codexis/project/files/a.docx"
    )
    assert (
        projects.harvest_path("smlouvy", "./dogs/buy-dog.docx")
        == "/home/codexis/project/files/smlouvy/dogs/buy-dog.docx"
    )


# ── scan + contract ──────────────────────────────────────────────────────────


def test_scan_relative_paths_and_checksums(project_files):
    files = projects.scan_files(os.path.join(project_files, "smlouvy"))
    assert [f["path"] for f in files] == ["./dogs/buy-dog.docx", "./readme.txt"]
    assert all(len(f["sha256"]) == 64 for f in files)


def test_scan_skips_hidden_entries(project_files):
    os.makedirs(os.path.join(project_files, ".hidden"))
    with open(os.path.join(project_files, ".hidden", "x.txt"), "w") as f:
        f.write("{}")
    paths = [f["path"] for f in projects.scan_files(project_files)]
    assert all(".hidden" not in p for p in paths)


def test_add_watch_scans_only_the_chosen_folder(watched_project, dokumenty_app_dir):
    res = projects.add_watch("Smlouvy", "smlouvy")
    assert res["projectId"] == watched_project and res["path"] == "smlouvy"
    assert res["files_available"] is True
    assert res["total"] == 2 and res["added"] == 2

    data = projects.read_watched(res["uuid"])
    assert data["version"] == projects.CONTRACT_VERSION
    assert data["projectId"] == watched_project and data["folder"] == "smlouvy"
    assert {d["path"] for d in data["documents"]} == {
        "./dogs/buy-dog.docx",
        "./readme.txt",
    }
    assert os.path.isfile(
        os.path.join(str(dokumenty_app_dir), "watched", res["uuid"], "watched.json")
    )


def test_add_watch_of_the_project_root_sees_everything(watched_project):
    res = projects.add_watch("Smlouvy")
    assert res["path"] == "."
    assert {d["path"] for d in projects.read_watched(res["uuid"])["documents"]} == {
        "./poznamky.md",
        "./smlouvy/dogs/buy-dog.docx",
        "./smlouvy/readme.txt",
    }


def test_add_watch_is_idempotent_per_folder(watched_project):
    first = projects.add_watch(watched_project, "smlouvy")
    again = projects.add_watch("Smlouvy", "./smlouvy/")
    root = projects.add_watch("Smlouvy")
    assert first["uuid"] == again["uuid"] != root["uuid"]
    assert len(projects.load_index()) == 2


def test_add_watch_without_visible_files_registers_an_empty_contract(
    tmp_path, monkeypatch, dokumenty_app_dir, daemon_projects
):
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    res = projects.add_watch("Audit", "faktury")
    assert res["files_available"] is False and res["total"] == 0
    assert projects.read_watched(res["uuid"])["documents"] == []
    assert projects.find_watch_of(OTHER_PROJECT_ID, "faktury")["name"] == "Audit"


def test_resolve_watch_finds_the_folder_of_a_project(watch):
    assert projects.resolve_watch("Smlouvy", "smlouvy")["uuid"] == watch
    with pytest.raises(WatchedProjectError):
        projects.resolve_watch("Smlouvy")
    with pytest.raises(WatchedProjectError):
        projects.resolve_watch("Audit", "smlouvy")


def test_sync_detects_new_changed_removed(watch, project_files):
    folder = os.path.join(project_files, "smlouvy")
    with open(os.path.join(folder, "dogs", "buy-dog.docx"), "w") as f:
        f.write("ZMENA 89/2012 Sb.")
    with open(os.path.join(folder, "new.txt"), "w") as f:
        f.write("novy")
    os.remove(os.path.join(folder, "readme.txt"))
    with open(os.path.join(project_files, "jinde.txt"), "w") as f:
        f.write("mimo sledovanou složku")

    _, summary = projects.sync_watched(watch)
    assert summary == {"added": 1, "changed": 1, "removed": 1, "total": 2}


def test_sync_fails_when_the_folder_vanished(watch, project_files):
    import shutil

    shutil.rmtree(os.path.join(project_files, "smlouvy"))
    with pytest.raises(WatchedProjectError):
        projects.sync_watched(watch)


def test_changed_checksum_resets_extraction(watch, project_files):
    projects.set_references(watch, "./readme.txt", [{"uri": "u", "text": "t"}])
    assert "./readme.txt" not in projects.docs_needing_harvest(watch)

    with open(os.path.join(project_files, "smlouvy", "readme.txt"), "w") as f:
        f.write("zcela jiny obsah")
    projects.sync_watched(watch)
    assert "./readme.txt" in projects.docs_needing_harvest(watch)


def test_set_references_stamps_timestamps(watch):
    doc = projects.set_references(
        watch,
        "./dogs/buy-dog.docx",
        [{"uri": "https://x", "text": "Zákon", "codexisId": "CR1"}],
    )
    assert doc["extractedAt"] is not None
    assert doc["legislation"][0]["codexisId"] == "CR1"


def test_unique_legislation_dedupes(watch):
    ref = [{"uri": "u", "text": "OZ", "codexisId": "CR26785"}]
    projects.set_references(watch, "./dogs/buy-dog.docx", ref)
    projects.set_references(watch, "./readme.txt", ref)
    uniq = projects.unique_legislation(watch)
    assert len(uniq) == 1 and uniq[0]["codexisId"] == "CR26785"


def test_mark_all_for_reharvest(watch):
    projects.set_references(watch, "./readme.txt", [{"uri": "u", "text": "t"}])
    assert projects.mark_all_for_reharvest(watch) == 1
    assert sorted(projects.docs_needing_harvest(watch)) == [
        "./dogs/buy-dog.docx",
        "./readme.txt",
    ]


def test_list_watches_counts(watch):
    projects.set_references(
        watch, "./readme.txt", [{"uri": "u", "text": "OZ", "codexisId": "CR26785"}]
    )
    [entry] = projects.list_watches()
    assert entry["uuid"] == watch and entry["path"] == "smlouvy"
    assert entry["documents"] == 2
    assert entry["pending_harvest"] == 1
    assert entry["legislation"] == 1
    assert entry["unconfirmed_changes"] == 0


def test_remove_watch_and_purge(watch):
    assert projects.remove_watch(watch) is True
    assert projects.load_index() == []
    assert projects.read_watched(watch) is not None
    assert projects.remove_watch(watch, purge=True) is False
    assert projects.read_watched(watch) is None


def test_require_available_needs_the_daemon(monkeypatch, dokumenty_app_dir):
    monkeypatch.setattr(_daemon, "graphql", lambda q, v=None, timeout=10: None)
    with pytest.raises(WatchedProjectError):
        projects.require_available()


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


def test_index_entry_without_a_folder_watches_the_root(dokumenty_app_dir):
    state.atomic_write_json(
        os.path.join(str(dokumenty_app_dir), "projects", PROJECT_ID, "watched.json"),
        {"version": 2, "projectId": PROJECT_ID, "documents": []},
    )
    projects.save_index([{"uuid": "old", "projectId": PROJECT_ID, "name": "Smlouvy"}])

    [entry] = projects.load_index()

    assert entry["path"] == "."
    assert projects.read_watched("old")["projectId"] == PROJECT_ID
    assert not os.path.exists(
        os.path.join(str(dokumenty_app_dir), "projects", PROJECT_ID)
    )
