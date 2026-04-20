"""Diff algorithms and URL/prompt builders for diff.py."""

from sledovane_dokumenty_core.diff import (
    build_compare_url,
    build_summary_prompt,
    unified_text_diff,
    word_level_changes,
)


class TestUnifiedTextDiff:
    def test_identical_returns_empty(self):
        assert unified_text_diff("same", "same") == ""

    def test_single_line_change(self):
        result = unified_text_diff("hello\n", "world\n")
        assert "-hello" in result
        assert "+world" in result

    def test_added_line(self):
        result = unified_text_diff("a\n", "a\nb\n")
        assert "+b" in result

    def test_removed_line(self):
        result = unified_text_diff("a\nb\n", "a\n")
        assert "-b" in result

    def test_label_appears_in_headers(self):
        result = unified_text_diff("a\n", "b\n", label="§1")
        assert "§1" in result


class TestWordLevelChanges:
    def test_identical_returns_empty(self):
        assert word_level_changes("same words here", "same words here") == ""

    def test_replace_marks_both_sides(self):
        result = word_level_changes("alpha bravo charlie", "alpha zulu charlie")
        assert "[-bravo-]" in result
        assert "[+zulu+]" in result

    def test_insert_marks_added_words(self):
        result = word_level_changes("alpha charlie", "alpha bravo charlie")
        assert "[+bravo+]" in result

    def test_delete_marks_removed_words(self):
        result = word_level_changes("alpha bravo charlie", "alpha charlie")
        assert "[-bravo-]" in result

    def test_context_words_limit_respected(self):
        # 20 words before, change, 20 words after; ctx=3 limits each side
        old = " ".join(f"w{i}" for i in range(20)) + " OLD " + " ".join(f"x{i}" for i in range(20))
        new = " ".join(f"w{i}" for i in range(20)) + " NEW " + " ".join(f"x{i}" for i in range(20))
        result = word_level_changes(old, new, context_words=3)
        # Context has 3 words on each side
        assert "w17 w18 w19" in result
        assert "w16" not in result  # 4th word back is outside context
        assert "x0 x1 x2" in result
        assert "x3" not in result

    def test_multiple_changes_separated_by_blank_line(self):
        old = "alpha bravo charlie delta echo foxtrot"
        new = "alpha ZULU charlie delta YANKEE foxtrot"
        result = word_level_changes(old, new, context_words=1)
        # Two replaces should produce two blocks separated by \n\n
        assert "\n\n" in result
        assert result.count("[-") == 2
        assert result.count("[+") == 2


class TestBuildCompareUrl:
    def test_contains_both_ids(self):
        url = build_compare_url("V1", "V2")
        assert "sourceDocId=V1" in url
        assert "targetDocId=V2" in url
        assert "puvodniZneni=V1" in url

    def test_fixed_parameters_present(self):
        url = build_compare_url("V1", "V2")
        assert "viewType=INSIDE" in url
        assert "changesOnly=true" in url

    def test_uses_next_codexis_host(self):
        assert build_compare_url("A", "B").startswith(
            "https://next.codexis.cz/porovnat"
        )


class TestBuildSummaryPrompt:
    def test_includes_doc_name(self):
        prompt = build_summary_prompt("Zákon č. 89/2012", [])
        assert "Zákon č. 89/2012" in prompt

    def test_mentions_old_new_sections(self):
        prompt = build_summary_prompt("Doc", [])
        assert "STARÝM" in prompt and "NOVÝM" in prompt

    def test_no_notes_omits_user_notes_block(self):
        prompt = build_summary_prompt("Doc", [])
        assert "zvláště zajímá" not in prompt

    def test_empty_notes_list_omits_user_notes_block(self):
        prompt = build_summary_prompt("Doc", [])
        assert "zvláště zajímá" not in prompt

    def test_notes_appended_with_semicolons(self):
        prompt = build_summary_prompt("Doc", ["DPH", "nájemní smlouvy", "dědictví"])
        assert "DPH; nájemní smlouvy; dědictví" in prompt
        assert "zvláště zajímá" in prompt

    def test_single_note_appended(self):
        prompt = build_summary_prompt("Doc", ["prekluze"])
        assert "prekluze" in prompt
        assert "zvláště zajímá" in prompt
