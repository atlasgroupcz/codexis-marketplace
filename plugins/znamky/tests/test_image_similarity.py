"""Figurative-similarity signals that work without Pillow: hashes + Vienna codes."""

from znamky_core import image_similarity as img


class TestHashSimilarity:
    def test_identical_hashes(self):
        h = {"phash": "ffffffffffffffff", "dhash": "0000000000000000"}
        assert img.hash_similarity(h, h) == 1.0

    def test_opposite_phash(self):
        a = {"phash": "0000000000000000"}
        b = {"phash": "ffffffffffffffff"}
        # 64 differing bits → similarity 0
        assert img.hash_similarity(a, b) == 0.0

    def test_missing_returns_none(self):
        assert img.hash_similarity(None, {"phash": "ff"}) is None
        assert img.hash_similarity({}, {}) is None


class TestColorSimilarity:
    def test_identical_vectors(self):
        v = [0.1, 0.2, 0.7]
        assert img.color_similarity(v, v) == 1.0

    def test_orthogonal_vectors(self):
        assert img.color_similarity([1.0, 0.0], [0.0, 1.0]) == 0.0

    def test_mismatched_length(self):
        assert img.color_similarity([1.0], [1.0, 0.0]) is None


class TestViennaOverlap:
    def test_exact_match(self):
        assert img.vienna_overlap(["26.4.1"], ["26.4.1"]) == 1.0

    def test_partial_hierarchy(self):
        # same category.division, different section → partial credit
        score = img.vienna_overlap(["26.4.1"], ["26.4.9"])
        assert 0.0 < score < 1.0

    def test_disjoint(self):
        assert img.vienna_overlap(["26.4.1"], ["01.1.1"]) == 0.0

    def test_empty(self):
        assert img.vienna_overlap([], ["26.4.1"]) == 0.0
