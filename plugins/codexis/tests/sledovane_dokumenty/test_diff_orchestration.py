"""diff.compute_version_diff and diff.build_change_text — orchestration layer.

These call clients.cdx_get_text to fetch actual document texts from CODEXIS, then
invoke the pure transformations tested in test_diff_normalize/sections/algo.
"""

import pytest

from sledovane_dokumenty_core import clients, diff


@pytest.fixture
def mock_cdx_text(monkeypatch):
    """Mock clients.cdx_get_text with configurable responses.

    Config keys:
      - texts_by_path: {path: str}
      - raise_for_paths: set[str] of paths that should raise CdxClientError
    """
    config = {"texts_by_path": {}, "raise_for_paths": set()}

    def _cdx_get_text(path):
        if path in config["raise_for_paths"]:
            raise clients.CdxClientError(f"fail for {path}")
        return config["texts_by_path"].get(path, "")

    monkeypatch.setattr(clients, "cdx_get_text", _cdx_get_text)
    return config


# ── compute_version_diff ─────────────────────────────────────────────────────


class TestComputeVersionDiffWholeDoc:
    def test_no_change_returns_empty(self, mock_cdx_text):
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text"] = "Same text"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text"] = "Same text"
        assert diff.compute_version_diff("v1", "v2", parts=[]) == []

    def test_text_change_yields_one_full_diff(self, mock_cdx_text):
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text"] = "Old text"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text"] = "New text"
        result = diff.compute_version_diff("v1", "v2", parts=[])
        assert len(result) == 1
        assert result[0]["part"] == "full"
        assert "Old" in result[0]["diff"] or "New" in result[0]["diff"]

    def test_fetch_failure_returns_empty(self, mock_cdx_text):
        mock_cdx_text["raise_for_paths"].add("cdx://doc/v1/text")
        assert diff.compute_version_diff("v1", "v2", parts=[]) == []

    def test_only_link_date_change_is_filtered(self, mock_cdx_text):
        # normalize_cdx_links strips dates → equal after normalize → no diff
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text"] = "See [z](cdx://doc/X1_2025_01_01)"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text"] = "See [z](cdx://doc/X1_2025_06_15)"
        assert diff.compute_version_diff("v1", "v2", parts=[]) == []


class TestComputeVersionDiffParts:
    def test_diffs_only_specified_parts(self, mock_cdx_text):
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text?part=p1"] = "Old p1"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text?part=p1"] = "New p1"
        result = diff.compute_version_diff("v1", "v2", parts=["p1"])
        assert len(result) == 1
        assert result[0]["part"] == "p1"

    def test_unchanged_parts_excluded(self, mock_cdx_text):
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text?part=a"] = "same"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text?part=a"] = "same"
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text?part=b"] = "old"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text?part=b"] = "new"
        result = diff.compute_version_diff("v1", "v2", parts=["a", "b"])
        assert [r["part"] for r in result] == ["b"]

    def test_client_error_skips_part_but_continues(self, mock_cdx_text):
        # First part fetch fails; second should still process
        mock_cdx_text["raise_for_paths"].add("cdx://doc/v1/text?part=broken")
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text?part=ok"] = "old"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text?part=ok"] = "new"
        result = diff.compute_version_diff("v1", "v2", parts=["broken", "ok"])
        assert [r["part"] for r in result] == ["ok"]

    def test_printer_called_with_progress(self, mock_cdx_text):
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text?part=p1"] = "same"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text?part=p1"] = "same"
        msgs = []
        diff.compute_version_diff("v1", "v2", parts=["p1"], printer=msgs.append)
        assert any("beze změn" in m for m in msgs)


# ── build_change_text ────────────────────────────────────────────────────────


class TestBuildChangeTextWholeDoc:
    def test_returns_none_on_fetch_error(self, mock_cdx_text):
        mock_cdx_text["raise_for_paths"].add("cdx://doc/v1/text")
        assert diff.build_change_text("v1", "v2", parts=[]) is None

    def test_uses_per_section_when_markers_present(self, mock_cdx_text):
        # new_text has [?part=...] markers → per_section_changes kicks in
        old = "[?part=intro]\nOld intro\n[?part=body]\nSame body"
        new = "[?part=intro]\nNew intro\n[?part=body]\nSame body"
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text"] = old
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text"] = new
        result = diff.build_change_text("v1", "v2", parts=[])
        assert "### intro" in result
        assert "Old intro" in result and "New intro" in result

    def test_falls_back_to_word_level_without_markers(self, mock_cdx_text):
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text"] = "alpha bravo charlie"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text"] = "alpha ZULU charlie"
        result = diff.build_change_text("v1", "v2", parts=[])
        assert "[-bravo-]" in result
        assert "[+ZULU+]" in result


class TestBuildChangeTextParts:
    def test_combined_parts_fed_to_word_level(self, mock_cdx_text):
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text?part=a"] = "alpha bravo"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text?part=a"] = "alpha ZULU"
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text?part=b"] = "gamma delta"
        mock_cdx_text["texts_by_path"]["cdx://doc/v2/text?part=b"] = "gamma YANKEE"
        result = diff.build_change_text("v1", "v2", parts=["a", "b"])
        assert "[-bravo-]" in result
        assert "[-delta-]" in result
        assert "[+ZULU+]" in result
        assert "[+YANKEE+]" in result

    def test_any_part_fetch_fail_returns_none(self, mock_cdx_text):
        mock_cdx_text["texts_by_path"]["cdx://doc/v1/text?part=a"] = "ok"
        mock_cdx_text["raise_for_paths"].add("cdx://doc/v1/text?part=b")
        assert diff.build_change_text("v1", "v2", parts=["a", "b"]) is None
