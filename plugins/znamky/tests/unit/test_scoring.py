"""Combined collision scoring: class overlap, weighting, class modulation, tiers."""

from znamky_core import scoring
from znamky_core.sources import base


def _candidate(text, classes=None, **kw):
    return base.make_candidate(
        source="TMVIEW",
        source_id="x",
        mark_text=text,
        mark_kind="word",
        nice_classes=classes or [],
        **kw,
    )


class TestClassOverlap:
    def test_partial(self):
        assert scoring.class_overlap([9, 42], [9]) == 0.5

    def test_disjoint(self):
        assert scoring.class_overlap([9], [30]) == 0.0

    def test_unknown_is_neutral(self):
        assert scoring.class_overlap([], [9]) == 0.5


class TestTier:
    def test_thresholds(self):
        assert scoring.tier_for(0.80) == "high"
        assert scoring.tier_for(0.70) == "medium"
        assert scoring.tier_for(0.50) == "low"


class TestScoreCandidate:
    def test_exact_same_class_is_high(self):
        watched = {"kind": "word", "text": "CODEXIS", "nice_classes": [9, 42]}
        result = scoring.score_candidate(watched, _candidate("CODEXIS", [9, 42]))
        assert result["scores"]["text_combined"] == 1.0
        assert result["scores"]["overall"] >= scoring.HIGH_THRESHOLD
        assert result["tier"] == "high"

    def test_same_text_unrelated_class_is_damped(self):
        watched = {"kind": "word", "text": "CODEXIS", "nice_classes": [9]}
        result = scoring.score_candidate(watched, _candidate("CODEXIS", [30]))
        # identical sign but unrelated goods → below the medium threshold, not zero
        assert result["scores"]["overall"] < scoring.MEDIUM_THRESHOLD
        assert result["scores"]["overall"] > 0.4

    def test_unrelated_text_is_low(self):
        watched = {"kind": "word", "text": "CODEXIS", "nice_classes": [9]}
        result = scoring.score_candidate(watched, _candidate("BANANA", [9]))
        assert result["tier"] == "low"
