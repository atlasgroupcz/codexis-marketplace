"""AI-harvest tests: citation parsing, resolution, and watched.json writes."""

from sledovane_dokumenty_core import clients, folders, harvest


def test_parse_cz_citation_detects_acts():
    assert harvest.parse_cz_citation("89/2012 Sb.") == ("89", "2012")
    assert harvest.parse_cz_citation("", "zákon č. 262/2006 Sb.") == ("262", "2006")


def test_parse_cz_citation_skips_eu():
    assert harvest.parse_cz_citation("Nařízení (EU) 2016/679") is None
    assert harvest.parse_cz_citation("Směrnice 2014/24/EU") is None


def test_build_entry_resolves_cz(monkeypatch):
    monkeypatch.setattr(
        clients, "resolve_cz_law",
        lambda num, year: "CR26785" if (num, year) == ("89", "2012") else None,
    )
    entry = harvest.build_legislation_entry({"text": "OZ", "citation": "89/2012 Sb.", "uri": ""})
    assert entry["uri"] == "https://www.zakonyprolidi.cz/cs/2012-89"
    assert entry["codexisId"] == "CR26785"
    assert entry["text"] == "OZ"


def test_build_entry_eu_keeps_model_uri(monkeypatch):
    monkeypatch.setattr(clients, "resolve_cz_law", lambda num, year: None)
    entry = harvest.build_legislation_entry(
        {"text": "GDPR", "citation": "Nařízení (EU) 2016/679", "uri": "https://eur-lex/x"}
    )
    assert "codexisId" not in entry
    assert entry["uri"] == "https://eur-lex/x"


def test_harvest_folder_writes_references(watched_root, monkeypatch):
    folders.add_watch(watched_root)

    def fake_extract(path, query, schema=None, schema_name=None):
        if path.endswith("buy-dog.docx"):
            return {"references": [{"text": "OZ", "citation": "89/2012 Sb.", "uri": ""}]}
        return {"references": []}

    monkeypatch.setattr(clients, "llm_extract_file", fake_extract)
    monkeypatch.setattr(clients, "resolve_cz_law", lambda num, year: "CR26785")

    res = harvest.harvest_folder(watched_root)
    assert res == {"documents": 2, "references": 1, "failed": 0}

    data = folders.read_watched(watched_root)
    doc = next(d for d in data["documents"] if d["path"] == "./dogs/buy-dog.docx")
    assert doc["extractedAt"] is not None
    assert doc["legislation"][0]["codexisId"] == "CR26785"
    assert folders.docs_needing_harvest(watched_root) == []


def test_harvest_failure_keeps_document_pending(watched_root, monkeypatch):
    folders.add_watch(watched_root)
    monkeypatch.setattr(clients, "llm_extract_file", lambda *a, **k: None)
    res = harvest.harvest_folder(watched_root)
    assert res["failed"] == 2 and res["documents"] == 0
    assert sorted(folders.docs_needing_harvest(watched_root)) == [
        "./dogs/buy-dog.docx",
        "./readme.txt",
    ]


def test_harvest_only_sends_pending_documents(watched_root, monkeypatch):
    folders.add_watch(watched_root)
    folders.set_references(watched_root, "./readme.txt", [{"uri": "u", "text": "t"}])

    calls = []

    def fake_extract(path, query, schema=None, schema_name=None):
        calls.append(path)
        return {"references": []}

    monkeypatch.setattr(clients, "llm_extract_file", fake_extract)
    harvest.harvest_folder(watched_root)
    assert len(calls) == 1 and calls[0].endswith("buy-dog.docx")
