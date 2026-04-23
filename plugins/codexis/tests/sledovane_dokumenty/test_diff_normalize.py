"""Text normalization for diff.py — pure regex functions."""

from sledovane_dokumenty_core.diff import (
    normalize_cdx_links,
    normalize_text,
    strip_changes_in_time,
    strip_part_markers,
)


class TestStripChangesInTime:
    def test_removes_single_block(self):
        text = "Before <changes_in_time>meta</changes_in_time> After"
        assert strip_changes_in_time(text) == "Before  After"

    def test_removes_multiline_block(self):
        text = (
            "Before\n"
            "<changes_in_time>\n"
            "line 1\n"
            "line 2\n"
            "</changes_in_time>\n"
            "After"
        )
        result = strip_changes_in_time(text)
        assert "line 1" not in result
        assert "line 2" not in result
        assert "Before" in result
        assert "After" in result

    def test_removes_multiple_blocks(self):
        text = (
            "A <changes_in_time>x</changes_in_time> "
            "B <changes_in_time>y</changes_in_time> C"
        )
        result = strip_changes_in_time(text)
        assert "x" not in result
        assert "y" not in result
        assert "A" in result and "B" in result and "C" in result

    def test_no_block_passes_through(self):
        assert strip_changes_in_time("plain text") == "plain text"

    def test_strips_surrounding_whitespace(self):
        assert strip_changes_in_time("  padded  ") == "padded"


class TestNormalizeCdxLinks:
    def test_strips_date_suffix(self):
        text = "See [link](cdx://doc/ZAK123_2025_04_01)"
        assert normalize_cdx_links(text) == "See [link](cdx://doc/ZAK123)"

    def test_multiple_links_all_stripped(self):
        text = "cdx://doc/A1_2025_04_01 and cdx://doc/B2_2024_12_31"
        assert normalize_cdx_links(text) == "cdx://doc/A1 and cdx://doc/B2"

    def test_link_without_date_unchanged(self):
        text = "cdx://doc/XYZ999 no date"
        assert normalize_cdx_links(text) == text

    def test_partial_date_not_stripped(self):
        # Must be full _YYYY_MM_DD shape
        text = "cdx://doc/XYZ_2025"
        assert normalize_cdx_links(text) == text

    def test_no_link_passes_through(self):
        assert normalize_cdx_links("no link here") == "no link here"


class TestStripPartMarkers:
    def test_removes_marker_line(self):
        text = "before\n[?part=abc]\nafter"
        result = strip_part_markers(text)
        assert "[?part=abc]" not in result
        assert "before" in result and "after" in result

    def test_removes_indented_marker_line(self):
        text = "before\n   [?part=abc]   \nafter"
        assert "[?part=abc]" not in strip_part_markers(text)

    def test_resolves_markdown_cdx_link_to_text(self):
        text = "See [tento zákon](cdx://doc/ZAK123)"
        assert strip_part_markers(text) == "See tento zákon"

    def test_removes_anchor_suffix(self):
        text = "See Section 5 (anchor in this document) for details"
        assert strip_part_markers(text) == "See Section 5 for details"

    def test_all_three_transformations_combined(self):
        text = (
            "[?part=intro]\n"
            "Viz [§5](cdx://doc/X1) (anchor in this document) dále"
        )
        result = strip_part_markers(text)
        assert "[?part=intro]" not in result
        assert "§5" in result
        assert "(cdx://" not in result
        assert "(anchor in this document)" not in result


class TestNormalizeText:
    def test_composes_all_three_stages(self):
        text = (
            "<changes_in_time>meta info</changes_in_time>\n"
            "[?part=intro]\n"
            "See [link](cdx://doc/X1_2025_04_01) (anchor in this document)"
        )
        result = normalize_text(text)
        assert "meta info" not in result
        assert "[?part=intro]" not in result
        assert "(cdx://" not in result
        assert "(anchor in this document)" not in result
        assert "link" in result

    def test_passes_through_plain_text(self):
        assert normalize_text("plain text") == "plain text"

    def test_link_date_in_markdown_link_resolved(self):
        # Date-suffixed link inside markdown should be resolved to text
        text = "See [zákon](cdx://doc/X1_2025_04_01)"
        result = normalize_text(text)
        assert result == "See zákon"
