"""Fixtures for the watched-project-folder tests."""

import pytest
from sledovane_dokumenty_core import _daemon, projects

PROJECT_ID = "5b1c1c4e-0000-4000-8000-000000000001"
OTHER_PROJECT_ID = "5b1c1c4e-0000-4000-8000-000000000002"


@pytest.fixture(autouse=True)
def fresh_chat_project_cache():
    """Each test is its own process as far as the memoized chat project goes."""
    projects._current_chat_project_cache.clear()
    yield
    projects._current_chat_project_cache.clear()


@pytest.fixture
def daemon_projects(monkeypatch):
    """A daemon that knows two projects and answers no other query.

    Tests that need a chat lookup override ``_daemon.graphql`` themselves.
    """

    def fake_graphql(query, variables=None, timeout=10):
        if query == projects._PROJECTS_QUERY:
            return {
                "projects": [
                    {
                        "id": projects.nodeids.encode(("Project", PROJECT_ID)),
                        "name": "Smlouvy",
                    },
                    {
                        "id": projects.nodeids.encode(("Project", OTHER_PROJECT_ID)),
                        "name": "Audit",
                    },
                ]
            }
        return {"node": None}

    monkeypatch.setattr(_daemon, "graphql", fake_graphql)
    monkeypatch.delenv("CODEXIS_PUBLIC_SESSION_ID", raising=False)


@pytest.fixture
def watched_project(tmp_path, monkeypatch, dokumenty_app_dir, daemon_projects):
    """A tmp project-home under a tmp home, with app state isolated.

    Lays the documents out where the daemon keeps a project's files on the raw
    VM, so ``projects.files_dir`` finds them the way the component's CGI does:
    two contracts under ``smlouvy/`` (one in a sub-folder) and a note at the
    root. Returns the project id.
    """
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))

    root = tmp_path / ".cdx" / "project-home" / PROJECT_ID / "files"
    (root / "smlouvy" / "dogs").mkdir(parents=True)
    (root / "smlouvy" / "dogs" / "buy-dog.docx").write_text(
        "smlouva dle 89/2012 Sb.", encoding="utf-8"
    )
    (root / "smlouvy" / "readme.txt").write_text(
        "odkaz na 262/2006 Sb.", encoding="utf-8"
    )
    (root / "smlouvy" / "photo.png").write_text("not a document", encoding="utf-8")
    (root / "poznamky.md").write_text("mimo sledovanou složku", encoding="utf-8")
    return PROJECT_ID


@pytest.fixture
def project_files(tmp_path, watched_project):
    """Absolute path of the watched project's files directory."""
    return str(tmp_path / ".cdx" / "project-home" / watched_project / "files")


@pytest.fixture
def watch(watched_project):
    """The ``smlouvy`` folder of the project, watched. Returns the watch id."""
    return projects.add_watch("Smlouvy", "smlouvy")["uuid"]
