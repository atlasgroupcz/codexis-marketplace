"""Verbal-similarity scoring: normalisation, metrics, Czech phonetic key, score()."""

from znamky_core import text_similarity as ts


class TestNormalize:
    def test_strips_accents_and_case(self):
        assert ts.normalize("Žluťoučký KŮŇ") == "zlutoucky kun"

    def test_collapses_punctuation_and_space(self):
        assert ts.normalize("  Co-dex!! is  ") == "co dex is"


class TestMetrics:
    def test_jaro_winkler_identical(self):
        assert ts.jaro_winkler("codexis", "codexis") == 1.0

    def test_jaro_winkler_prefix_boost(self):
        # shared prefix "codex" → high score
        assert ts.jaro_winkler("codexis", "codexus") > 0.9

    def test_levenshtein_ratio(self):
        assert ts.levenshtein_ratio("abc", "abc") == 1.0
        assert ts.levenshtein_ratio("abc", "abd") == 1 - 1 / 3
        assert ts.levenshtein_ratio("", "") == 1.0

    def test_trigram_jaccard_disjoint(self):
        assert ts.trigram_jaccard("aaaa", "zzzz") == 0.0


class TestPhonetic:
    def test_c_k_collapse(self):
        # "Codexis" and "Kodexis" should share a phonetic skeleton
        assert ts.czech_phonetic_key("Codexis") == ts.czech_phonetic_key("Kodexis")

    def test_w_v_equivalence(self):
        assert ts.czech_phonetic_key("Wolt") == ts.czech_phonetic_key("Volt")


class TestScore:
    def test_identical_is_one(self):
        s = ts.score("CODEXIS", "codexis")
        assert s["combined"] == 1.0

    def test_phonetic_variant_is_high(self):
        s = ts.score("CODEXIS", "KODEXIS")
        assert s["combined"] >= 0.75

    def test_containment_boost(self):
        s = ts.score("CODEX", "CODEX PLUS")
        assert s["orthographic"] >= 0.85

    def test_unrelated_is_low(self):
        s = ts.score("CODEXIS", "BANANA")
        assert s["combined"] < 0.4

    def test_empty_is_zero(self):
        assert ts.score("", "x")["combined"] == 0.0
