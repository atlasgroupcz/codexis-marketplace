"""Fixtures for the watched-folders tests."""

import pytest

from sledovane_dokumenty_core import folders


@pytest.fixture
def watched_root(tmp_path, monkeypatch, dokumenty_app_dir):
    """A tmp document tree under a tmp home, with app state isolated.

    `dokumenty_app_dir` (from the parent conftest) redirects state.APP_DIR, so the
    watched-folders index + settings land in the tmp app dir. The automation
    bootstrap is stubbed so tests never shell out to cdxctl.
    """
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    monkeypatch.setattr(folders, "ensure_folder_check_automation", lambda: None)

    root = tmp_path / "contracts"
    (root / "dogs").mkdir(parents=True)
    (root / "dogs" / "buy-dog.docx").write_text("smlouva dle 89/2012 Sb.", encoding="utf-8")
    (root / "readme.txt").write_text("odkaz na 262/2006 Sb.", encoding="utf-8")
    (root / "photo.png").write_text("not a document", encoding="utf-8")
    return str(root)
