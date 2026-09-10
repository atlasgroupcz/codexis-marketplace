"""The watched.json file must match the documented public contract."""

from sledovane_dokumenty_core import projects

DOC_KEYS = {"path", "sha256", "discoveredAt", "extractedAt", "updatedAt", "legislation"}


def test_watched_json_matches_documented_contract(watched_project):
    projects.add_watch(watched_project)
    projects.set_references(
        watched_project,
        "./dogs/buy-dog.docx",
        [
            {
                "uri": "https://www.zakonyprolidi.cz/cs/2012-89",
                "text": "OZ",
                "codexisId": "CR26785",
            }
        ],
    )
    data = projects.read_watched(watched_project)

    assert set(data.keys()) >= {"version", "projectId", "generatedAt", "documents"}
    assert isinstance(data["version"], int)
    assert data["projectId"] == watched_project

    for doc in data["documents"]:
        assert set(doc.keys()) == DOC_KEYS
        assert doc["path"].startswith("./")
        assert isinstance(doc["sha256"], str) and len(doc["sha256"]) == 64
        assert isinstance(doc["legislation"], list)

    harvested = next(d for d in data["documents"] if d["path"] == "./dogs/buy-dog.docx")
    leg = harvested["legislation"][0]
    assert set(leg.keys()) >= {"uri", "text"}
    assert leg["uri"].startswith("https://")
