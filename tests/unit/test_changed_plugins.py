from pathlib import Path
from unittest.mock import patch
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from _changed_plugins import get_changed_plugins, CORE_TRIGGERS


class FakeRepo:
    """Represents a repo with certain plugin directories existing."""
    def __init__(self, tmp_path: Path, plugins: list[str]):
        self.root = tmp_path
        (tmp_path / ".claude-plugin").mkdir()
        (tmp_path / ".claude-plugin" / "marketplace.json").write_text("{}")
        for p in plugins:
            d = tmp_path / "plugins" / p / "acceptance" / "e2e"
            d.mkdir(parents=True)
            (d / "foo.yaml").write_text("name: foo\nsteps: []\n")


def test_detects_single_changed_plugin(tmp_path):
    FakeRepo(tmp_path, ["codexis", "katastr"])
    with patch("_changed_plugins._git_diff_files",
               return_value=["plugins/codexis/skills/codexis/SKILL.md"]):
        assert get_changed_plugins("main", tmp_path) == ["codexis"]


def test_detects_multiple_changed_plugins(tmp_path):
    FakeRepo(tmp_path, ["codexis", "katastr", "ocr"])
    with patch("_changed_plugins._git_diff_files",
               return_value=[
                   "plugins/codexis/SKILL.md",
                   "plugins/ocr/bin/x",
                   "README.md",
               ]):
        assert get_changed_plugins("main", tmp_path) == ["codexis", "ocr"]


def test_core_trigger_runs_all_plugins_with_e2e(tmp_path):
    FakeRepo(tmp_path, ["codexis", "katastr"])
    # Plugin without acceptance/e2e should NOT be in "all" set.
    (tmp_path / "plugins" / "bare").mkdir(parents=True)
    with patch("_changed_plugins._git_diff_files",
               return_value=[".claude-plugin/marketplace.json"]):
        assert get_changed_plugins("main", tmp_path) == ["codexis", "katastr"]


def test_deleted_plugin_dir_is_ignored(tmp_path):
    FakeRepo(tmp_path, ["codexis"])
    with patch("_changed_plugins._git_diff_files",
               return_value=["plugins/removed-plugin/SKILL.md"]):
        assert get_changed_plugins("main", tmp_path) == []


def test_core_triggers_list_is_explicit():
    # Guard against accidental wildcarding of core-trigger detection.
    assert ".claude-plugin/marketplace.json" in CORE_TRIGGERS
    assert "tests/test-plugin-e2e.py" in CORE_TRIGGERS
    assert "pyproject.toml" in CORE_TRIGGERS


def test_empty_diff_returns_empty(tmp_path):
    FakeRepo(tmp_path, ["codexis"])
    with patch("_changed_plugins._git_diff_files", return_value=[]):
        assert get_changed_plugins("main", tmp_path) == []
