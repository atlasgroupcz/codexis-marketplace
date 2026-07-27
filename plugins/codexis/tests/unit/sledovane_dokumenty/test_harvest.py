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


def test_build_entry_keeps_model_uri_when_no_source_recognises_the_citation(monkeypatch):
    # No "Sb." number and no EU act type, so neither resolver can key it.
    monkeypatch.setattr(clients, "resolve_cz_law", lambda num, year: None)
    monkeypatch.setattr(clients, "resolve_eu_law", lambda letter, year, number: None)
    entry = harvest.build_legislation_entry(
        {"text": "Sdělení Komise", "citation": "Sdělení Komise 2016/123", "uri": "https://model/x"}
    )
    assert "codexisId" not in entry
    assert entry["uri"] == "https://model/x"


def test_build_entry_eu_unresolved_keeps_no_codexis_id(monkeypatch):
    monkeypatch.setattr(clients, "resolve_cz_law", lambda num, year: None)
    monkeypatch.setattr(clients, "resolve_eu_law", lambda letter, year, number: None)
    entry = harvest.build_legislation_entry(
        {"text": "GDPR", "citation": "Nařízení (EU) 2016/679", "uri": "https://eur-lex/x"}
    )
    assert "codexisId" not in entry


def test_parse_eu_citation_reads_year_first_numbering():
    # Acts from 2015 on are cited YEAR/NUMBER.
    assert harvest.parse_eu_citation("Nařízení (EU) 2016/679") == ("R", "2016", "679")


def test_parse_eu_citation_reads_number_first_numbering():
    # Older acts are cited NUMBER/YEAR.
    assert harvest.parse_eu_citation("Nařízení (ES) č. 988/2009") == ("R", "2009", "988")


def test_parse_eu_citation_maps_act_type_to_sector_letter():
    assert harvest.parse_eu_citation("Směrnice 2014/24/EU") == ("L", "2014", "24")
    assert harvest.parse_eu_citation("Rozhodnutí (EU) 2015/2240") == ("D", "2015", "2240")


def test_parse_eu_citation_strips_padding_so_one_act_keys_one_lookup():
    # A citation may write the number padded; both spellings are the same act and
    # must not resolve twice.
    assert harvest.parse_eu_citation("Nařízení (EU) 2016/0679") == ("R", "2016", "679")


def test_parse_eu_citation_returns_none_for_unknown_act_type():
    assert harvest.parse_eu_citation("Sdělení Komise 2016/123") is None


def test_eu_resolution_yields_no_id_when_the_api_call_fails(monkeypatch):
    # Any failed get (a 404 while the route is undeployed, a network error) makes
    # cdx-cli exit non-zero, which surfaces as CdxClientError. The act must stay
    # unresolved rather than break the harvest.
    def failing_get(path):
        raise clients.CdxClientError(f"cdx-cli get failed for {path}")

    clients.resolve_eu_law.cache_clear()
    monkeypatch.setattr(clients, "cdx_get", failing_get)
    assert clients.resolve_eu_law("R", "2016", "679") is None


def test_eu_resolution_calls_the_api_once_per_act(monkeypatch):
    calls = []

    def counting_get(path):
        calls.append(path)
        return {"docId": "EU133150", "eu": {"celex": "32016R0679"}}

    clients.resolve_eu_law.cache_clear()
    monkeypatch.setattr(clients, "cdx_get", counting_get)

    assert clients.resolve_eu_law("R", "2016", "679") == ("EU133150", "32016R0679")
    assert clients.resolve_eu_law("R", "2016", "679") == ("EU133150", "32016R0679")
    assert len(calls) == 1


def test_eu_entry_gets_the_canonical_eur_lex_uri(monkeypatch):
    monkeypatch.setattr(clients, "resolve_cz_law", lambda num, year: None)
    monkeypatch.setattr(clients, "resolve_eu_law", lambda *parts: ("EU133150", "32016R0679"))
    entry = harvest.build_legislation_entry(
        {"text": "GDPR", "citation": "Nařízení (EU) 2016/679", "uri": "https://model-guess"}
    )
    assert entry["uri"] == (
        "https://eur-lex.europa.eu/legal-content/CS/TXT/?uri=CELEX:32016R0679"
    )


def test_build_entry_resolves_eu_by_citation_coordinates(monkeypatch):
    monkeypatch.setattr(clients, "resolve_cz_law", lambda num, year: None)
    monkeypatch.setattr(
        clients, "resolve_eu_law",
        lambda *parts: ("EU133150", "32016R0679") if parts == ("R", "2016", "679") else None,
    )
    entry = harvest.build_legislation_entry(
        {"text": "GDPR", "citation": "Nařízení (EU) 2016/679", "uri": "https://eur-lex/x"}
    )
    assert entry["codexisId"] == "EU133150"


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
