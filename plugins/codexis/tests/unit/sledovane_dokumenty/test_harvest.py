"""AI-harvest tests: citation parsing, resolution, and watched.json writes."""

from sledovane_dokumenty_core import clients, harvest


def test_parse_cz_citation_detects_acts():
    assert harvest.parse_cz_citation("89/2012 Sb.") == ("89", "2012")
    assert harvest.parse_cz_citation("", "zákon č. 262/2006 Sb.") == ("262", "2006")


def test_parse_cz_citation_skips_eu():
    assert harvest.parse_cz_citation("Nařízení (EU) 2016/679") is None
    assert harvest.parse_cz_citation("Směrnice 2014/24/EU") is None


def test_build_entry_resolves_cz(monkeypatch):
    monkeypatch.setattr(
        clients,
        "resolve_cz_law",
        lambda num, year: "CR26785" if (num, year) == ("89", "2012") else None,
    )
    entry = harvest.build_legislation_entry(
        {"text": "OZ", "citation": "89/2012 Sb.", "uri": ""}
    )
    assert entry["uri"] == "https://www.zakonyprolidi.cz/cs/2012-89"
    assert entry["codexisId"] == "CR26785"
    assert entry["text"] == "OZ"


def test_build_entry_keeps_model_uri_when_no_source_recognises_the_citation(
    monkeypatch,
):
    # No "Sb." number, so the Czech resolver cannot key it.
    monkeypatch.setattr(clients, "resolve_cz_law", lambda num, year: None)
    entry = harvest.build_legislation_entry(
        {
            "text": "Sdělení Komise",
            "citation": "Sdělení Komise 2016/123",
            "uri": "https://model/x",
        }
    )
    assert "codexisId" not in entry
    assert entry["uri"] == "https://model/x"


def test_build_entry_leaves_an_eu_citation_untouched(monkeypatch):
    # Nothing resolves an EU act - the API has no route for it - so the entry keeps
    # whatever URI the model produced and carries no id.
    def fail_if_called(num, year):
        raise AssertionError(f"Czech resolver reached for an EU act: {num}/{year}")

    monkeypatch.setattr(clients, "resolve_cz_law", fail_if_called)
    entry = harvest.build_legislation_entry(
        {
            "text": "GDPR",
            "citation": "Nařízení (EU) 2016/679",
            "uri": "https://eur-lex/x",
        }
    )
    assert "codexisId" not in entry
    assert entry["uri"] == "https://eur-lex/x"


# ── harvesting a watched folder ──────────────────────────────────────────────


def _extract_ok(monkeypatch, seen):
    def fake_extract(file_path, query, schema=None, schema_name=None, session_id=None):
        seen.append((file_path, session_id))
        return {"references": [{"text": "OZ", "citation": "89/2012 Sb.", "uri": ""}]}

    monkeypatch.setattr(clients, "llm_extract_file", fake_extract)
    monkeypatch.setattr(clients, "resolve_cz_law", lambda num, year: "CR26785")


def test_harvest_watch_reads_documents_through_a_chat_of_the_project(
    watch, monkeypatch
):
    from sledovane_dokumenty_core import projects

    monkeypatch.setattr(projects, "harvest_session", lambda pid: "chat-1")
    seen = []
    _extract_ok(monkeypatch, seen)

    res = harvest.harvest_watch(watch)

    assert res == {"documents": 2, "references": 2, "failed": 0}
    assert seen == [
        ("/home/codexis/project/files/smlouvy/dogs/buy-dog.docx", "chat-1"),
        ("/home/codexis/project/files/smlouvy/readme.txt", "chat-1"),
    ]
    assert projects.docs_needing_harvest(watch) == []
    doc = next(
        d
        for d in projects.read_watched(watch)["documents"]
        if d["path"] == "./readme.txt"
    )
    assert doc["legislation"][0]["codexisId"] == "CR26785"


def test_harvest_watch_scans_first_when_the_files_are_visible(
    watch, project_files, monkeypatch
):
    import os

    from sledovane_dokumenty_core import projects

    with open(os.path.join(project_files, "smlouvy", "new.txt"), "w") as f:
        f.write("odkaz na 89/2012 Sb.")
    monkeypatch.setattr(projects, "harvest_session", lambda pid: "chat-1")
    seen = []
    _extract_ok(monkeypatch, seen)

    res = harvest.harvest_watch(watch)

    assert res["documents"] == 3
    assert "/home/codexis/project/files/smlouvy/new.txt" in [p for p, _ in seen]


def test_harvest_watch_needs_a_chat_in_the_project(watch, monkeypatch):
    import pytest
    from sledovane_dokumenty_core import projects
    from sledovane_dokumenty_core.projects import WatchedProjectError

    monkeypatch.setattr(projects, "harvest_session", lambda pid: None)
    monkeypatch.setattr(
        clients, "llm_extract_file", lambda *a, **k: pytest.fail("extract reached")
    )

    with pytest.raises(WatchedProjectError):
        harvest.harvest_watch(watch)


def test_harvest_watch_leaves_failed_documents_pending(watch, monkeypatch):
    from sledovane_dokumenty_core import projects

    monkeypatch.setattr(projects, "harvest_session", lambda pid: "chat-1")
    monkeypatch.setattr(clients, "llm_extract_file", lambda *a, **k: None)

    res = harvest.harvest_watch(watch)

    assert res == {"documents": 0, "references": 0, "failed": 2}
    assert len(projects.docs_needing_harvest(watch)) == 2
