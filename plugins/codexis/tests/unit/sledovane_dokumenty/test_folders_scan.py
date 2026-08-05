"""Scan + watched.json contract tests for sledovane_dokumenty_core.folders."""

import os

import pytest
from sledovane_dokumenty_core import folders
from sledovane_dokumenty_core.folders import WatchedFolderError


def test_scan_relative_paths_and_checksums(watched_root):
    files = folders.scan_files(watched_root)
    # photo.png excluded (unsupported), results sorted by relative path
    assert [f["path"] for f in files] == ["./dogs/buy-dog.docx", "./readme.txt"]
    assert all(len(f["sha256"]) == 64 for f in files)


def test_scan_skips_hidden_watched_dir(watched_root):
    os.makedirs(os.path.join(watched_root, ".watched"))
    with open(os.path.join(watched_root, ".watched", "watched.json"), "w") as f:
        f.write("{}")
    paths = [f["path"] for f in folders.scan_files(watched_root)]
    assert all(".watched" not in p for p in paths)


def test_add_watch_writes_contract(watched_root):
    res = folders.add_watch(watched_root)
    assert res["total"] == 2 and res["added"] == 2

    data = folders.read_watched(watched_root)
    assert data["version"] == folders.CONTRACT_VERSION
    assert data["watchedRoot"] == "."
    assert data["generatedAt"]
    assert {d["path"] for d in data["documents"]} == {
        "./dogs/buy-dog.docx",
        "./readme.txt",
    }

    doc = next(d for d in data["documents"] if d["path"] == "./readme.txt")
    assert doc["extractedAt"] is None
    assert doc["discoveredAt"] and doc["updatedAt"]
    assert doc["legislation"] == []
    assert len(doc["sha256"]) == 64

    # stored in the hidden .watched folder
    assert os.path.isfile(os.path.join(watched_root, ".watched", "watched.json"))


def test_add_watch_registers_index(watched_root):
    folders.add_watch(watched_root, name="Smlouvy")
    index = folders.load_index()
    assert len(index) == 1
    assert index[0]["root"] == os.path.abspath(watched_root)
    assert index[0]["name"] == "Smlouvy"
    # idempotent on the same root
    folders.add_watch(watched_root)
    assert len(folders.load_index()) == 1


def test_sync_detects_new_changed_removed(watched_root):
    folders.add_watch(watched_root)

    # change one file, add one, remove one
    with open(os.path.join(watched_root, "dogs", "buy-dog.docx"), "w") as f:
        f.write("ZMENA 89/2012 Sb.")
    with open(os.path.join(watched_root, "new.txt"), "w") as f:
        f.write("novy")
    os.remove(os.path.join(watched_root, "readme.txt"))

    _, summary = folders.sync_watched(watched_root)
    assert summary == {"added": 1, "changed": 1, "removed": 1, "total": 2}


def test_changed_checksum_resets_extraction(watched_root):
    folders.add_watch(watched_root)
    folders.set_references(watched_root, "./readme.txt", [{"uri": "u", "text": "t"}])
    assert "./readme.txt" not in folders.docs_needing_harvest(watched_root)

    with open(os.path.join(watched_root, "readme.txt"), "w") as f:
        f.write("zcela jiny obsah")
    folders.sync_watched(watched_root)
    assert "./readme.txt" in folders.docs_needing_harvest(watched_root)


def test_set_references_stamps_timestamps(watched_root):
    folders.add_watch(watched_root)
    doc = folders.set_references(
        watched_root,
        "./dogs/buy-dog.docx",
        [{"uri": "https://x", "text": "Zákon", "codexisId": "CR1"}],
    )
    assert doc["extractedAt"] is not None
    assert doc["legislation"][0]["codexisId"] == "CR1"


def test_unique_legislation_dedupes(watched_root):
    folders.add_watch(watched_root)
    folders.set_references(
        watched_root,
        "./dogs/buy-dog.docx",
        [{"uri": "u", "text": "OZ", "codexisId": "CR26785"}],
    )
    folders.set_references(
        watched_root,
        "./readme.txt",
        [{"uri": "u", "text": "OZ", "codexisId": "CR26785"}],
    )
    uniq = folders.unique_legislation(watched_root)
    assert len(uniq) == 1 and uniq[0]["codexisId"] == "CR26785"


def test_mark_all_for_reharvest(watched_root):
    folders.add_watch(watched_root)
    folders.set_references(watched_root, "./readme.txt", [{"uri": "u", "text": "t"}])
    count = folders.mark_all_for_reharvest(watched_root)
    assert count == 1
    assert sorted(folders.docs_needing_harvest(watched_root)) == [
        "./dogs/buy-dog.docx",
        "./readme.txt",
    ]


def test_remove_watch(watched_root):
    folders.add_watch(watched_root)
    assert folders.remove_watch(watched_root) is True
    assert folders.load_index() == []
    assert folders.remove_watch(watched_root) is False


def test_normalize_root_rejects_escape(tmp_path, monkeypatch):
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path / "home"))
    (tmp_path / "home").mkdir()
    with pytest.raises(WatchedFolderError):
        folders.normalize_root(str(tmp_path / "elsewhere"))


def test_browse_lists_subdirs_and_refuses_escape(tmp_path, monkeypatch):
    home = tmp_path / "home"
    (home / "a").mkdir(parents=True)
    (home / "b").mkdir()
    (home / ".hidden").mkdir()
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(home))

    result = folders.browse(str(home))
    names = [e["name"] for e in result["entries"]]
    assert names == ["a", "b"]  # hidden excluded, sorted
    assert result["home"] == str(home)
    assert result["parent"] is None  # at home root

    # escaping above home falls back to home
    assert folders.browse(str(tmp_path))["path"] == str(home)
