"""Section extraction and per-section diff for diff.py."""

from sledovane_dokumenty_core.diff import (
    extract_part_ids,
    extract_section,
    per_section_changes,
)


class TestExtractSection:
    def test_returns_content_between_markers(self):
        text = (
            "[?part=intro]\n"
            "This is the intro.\n"
            "[?part=body]\n"
            "This is the body."
        )
        assert extract_section(text, "intro") == "This is the intro."

    def test_returns_last_section_until_end(self):
        text = (
            "[?part=intro]\n"
            "Intro text.\n"
            "[?part=body]\n"
            "Body line 1.\n"
            "Body line 2."
        )
        result = extract_section(text, "body")
        assert "Body line 1." in result
        assert "Body line 2." in result
        assert "Intro" not in result

    def test_missing_marker_returns_empty(self):
        text = "[?part=intro]\ncontent"
        assert extract_section(text, "nonexistent") == ""

    def test_empty_section_returns_empty(self):
        text = "[?part=empty]\n[?part=next]\ncontent"
        assert extract_section(text, "empty") == ""

    def test_strips_surrounding_whitespace(self):
        text = "[?part=s]\n\n   content   \n\n[?part=next]"
        assert extract_section(text, "s") == "content"

    def test_indented_marker_still_matches(self):
        text = "   [?part=intro]\ntext"
        assert extract_section(text, "intro") == "text"


class TestExtractPartIds:
    def test_returns_all_ids(self):
        text = "[?part=intro]\nA\n[?part=body]\nB\n[?part=tail]\nC"
        assert extract_part_ids(text) == ["intro", "body", "tail"]

    def test_empty_when_no_markers(self):
        assert extract_part_ids("plain text") == []

    def test_preserves_duplicate_ids(self):
        text = "[?part=x]\nA\n[?part=x]\nB"
        assert extract_part_ids(text) == ["x", "x"]

    def test_indented_markers_recognized(self):
        text = "   [?part=intro]\ncontent"
        assert extract_part_ids(text) == ["intro"]

    def test_ignores_inline_markers(self):
        # Marker must be at start of line; inline occurrence should not count
        text = "A line with [?part=inline] inside."
        assert extract_part_ids(text) == []


class TestPerSectionChanges:
    def test_none_when_no_markers(self):
        assert per_section_changes("old", "new") is None

    def test_none_when_sections_unchanged(self):
        text = "[?part=intro]\nSame text"
        assert per_section_changes(text, text) is None

    def test_returns_sections_that_differ(self):
        old = (
            "[?part=intro]\n"
            "Old intro.\n"
            "[?part=body]\n"
            "Same body."
        )
        new = (
            "[?part=intro]\n"
            "New intro.\n"
            "[?part=body]\n"
            "Same body."
        )
        result = per_section_changes(old, new)
        assert result is not None
        assert "### intro" in result
        assert "Old intro." in result
        assert "New intro." in result
        assert "### body" not in result

    def test_multiple_changed_sections(self):
        old = "[?part=a]\nold-a\n[?part=b]\nold-b"
        new = "[?part=a]\nnew-a\n[?part=b]\nnew-b"
        result = per_section_changes(old, new)
        assert result is not None
        assert "### a" in result and "### b" in result
        assert "old-a" in result and "new-a" in result

    def test_normalizes_before_comparing(self):
        # Link date differs but after normalize_cdx_links they're equal → no change
        old = "[?part=x]\nsee [z](cdx://doc/A1_2025_01_01)"
        new = "[?part=x]\nsee [z](cdx://doc/A1_2025_06_15)"
        assert per_section_changes(old, new) is None

    def test_skips_changes_in_time_blocks(self):
        # Changes in <changes_in_time> are normalized away → no real change
        old = "[?part=x]\n<changes_in_time>meta1</changes_in_time>\ntext"
        new = "[?part=x]\n<changes_in_time>meta2</changes_in_time>\ntext"
        assert per_section_changes(old, new) is None
