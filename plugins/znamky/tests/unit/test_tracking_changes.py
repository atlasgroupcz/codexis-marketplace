"""Watch-state lifecycle: baseline, new-collision detection, confirm, assessment, sort."""

import json
import os

import pytest
from znamky_core import tracking
from znamky_core.sources import base


def _cand(text, source_id, classes=(9, 42), **kw):
    return base.make_candidate(
        source="TMVIEW",
        source_id=source_id,
        mark_text=text,
        mark_kind="word",
        nice_classes=list(classes),
        office="CZ",
        **kw,
    )


def _patch_search(monkeypatch, candidates, errors=None):
    monkeypatch.setattr(
        tracking,
        "_run_search",
        lambda state: {"candidates": list(candidates), "errors": list(errors or [])},
    )


def _seed(app_dir, uuid, **overrides):
    base_state = {
        "uuid": uuid,
        "kind": "word",
        "label": "",
        "text": f"MARK-{uuid}",
        "nice_classes": [9],
        "territories": ["CZ", "EU"],
        "threshold": 0.6,
        "added_on": "2026-04-01T00:00:00Z",
        "last_check_at": "2026-04-01T00:00:00Z",
        "known_collision_ids": [],
        "collisions": [],
    }
    base_state.update(overrides)
    dpath = os.path.join(str(app_dir), uuid)
    os.makedirs(dpath, exist_ok=True)
    with open(os.path.join(dpath, "state.json"), "w", encoding="utf-8") as f:
        json.dump(base_state, f)
    return base_state


class TestAddAndBaseline:
    def test_add_text_baselines_current_conflicts(self, app_dir, monkeypatch):
        _patch_search(
            monkeypatch,
            [
                _cand("CODEXIS", "id-exact"),  # ~identical → above threshold
                _cand("BANANA", "id-far"),  # unrelated → below threshold
            ],
        )
        state = tracking.add_text("CODEXIS", nice_classes="9,42", territories="CZ,EU")
        ids = [c["source_id"] for c in state["collisions"]]
        assert "id-exact" in ids
        assert "id-far" not in ids
        # all baseline collisions are unconfirmed and seeded into known ids
        assert all(c["confirmed_on"] is None for c in state["collisions"])
        assert any("id-exact" in k for k in state["known_collision_ids"])


class TestCheckDetectsNew:
    def test_only_new_marks_alert(self, app_dir, monkeypatch):
        _patch_search(monkeypatch, [_cand("CODEXIS", "id-exact")])
        state = tracking.add_text("CODEXIS", nice_classes="9,42")
        assert len(state["collisions"]) == 1

        # next check: same mark + a new similar one
        _patch_search(
            monkeypatch,
            [
                _cand("CODEXIS", "id-exact"),
                _cand("KODEXIS", "id-new"),
            ],
        )
        result = tracking.check_one(state["uuid"])
        assert result["ok"]
        new_ids = [c["source_id"] for c in result["new_collisions"]]
        assert new_ids == ["id-new"]  # only the new one alerts
        refreshed = tracking.find_by_uuid(state["uuid"])
        assert refreshed is not None
        assert len(refreshed["collisions"]) == 2  # prepended to history

    def test_search_error_is_reported_not_raised(self, app_dir, monkeypatch):
        _patch_search(monkeypatch, [_cand("CODEXIS", "id-exact")])
        state = tracking.add_text("CODEXIS", nice_classes="9,42")
        _patch_search(monkeypatch, [], errors=[{"source": "TMVIEW", "error": "down"}])
        result = tracking.check_one(state["uuid"])
        assert result["ok"]
        assert "down" in (result["error"] or "")


class TestConfirmAndAssess:
    def test_confirm_all(self, app_dir, monkeypatch):
        _patch_search(monkeypatch, [_cand("CODEXIS", "id-exact")])
        state = tracking.add_text("CODEXIS", nice_classes="9,42")
        assert tracking.confirm(state["uuid"]) == 1
        assert tracking.confirm(state["uuid"]) == 0  # idempotent

    def test_set_assessment(self, app_dir, monkeypatch):
        _patch_search(monkeypatch, [_cand("CODEXIS", "id-exact")])
        state = tracking.add_text("CODEXIS", nice_classes="9,42")
        collision = tracking.set_assessment(
            state["uuid"],
            0,
            {"risk": "high", "visual": "v", "summary": "s"},
        )
        assert collision["ai_assessment"]["risk"] == "high"

    def test_set_assessment_rejects_bad_risk(self, app_dir, monkeypatch):
        _patch_search(monkeypatch, [_cand("CODEXIS", "id-exact")])
        state = tracking.add_text("CODEXIS", nice_classes="9,42")
        with pytest.raises(tracking.InvalidMarkError):
            tracking.set_assessment(state["uuid"], 0, {"risk": "catastrophic"})


class TestValidation:
    def test_bad_nice_class(self, app_dir, monkeypatch):
        _patch_search(monkeypatch, [])
        with pytest.raises(tracking.InvalidMarkError):
            tracking.add_text("X", nice_classes="99")

    def test_bad_territory(self, app_dir, monkeypatch):
        _patch_search(monkeypatch, [])
        with pytest.raises(tracking.InvalidMarkError):
            tracking.add_text("X", territories="US")


class TestAddTextDedup:
    def test_duplicate_add_text_raises_and_keeps_one(self, app_dir, monkeypatch):
        _patch_search(monkeypatch, [])
        tracking.add_text("CODEXIS", nice_classes="9,42", territories="CZ,EU")
        with pytest.raises(tracking.MarkAlreadyTrackedError):
            # reordered classes + territories prove the key is order-insensitive
            tracking.add_text("CODEXIS", nice_classes="42,9", territories="EU,CZ")
        assert len(tracking.list_all()) == 1


class TestListSort:
    def test_unconfirmed_high_first(self, app_dir):
        _seed(app_dir, "idle", collisions=[])
        _seed(
            app_dir,
            "reviewed",
            collisions=[
                {
                    "tier": "high",
                    "detected_on": "2026-04-10T00:00:00Z",
                    "confirmed_on": "2026-04-11T00:00:00Z",
                },
            ],
        )
        _seed(
            app_dir,
            "hot",
            collisions=[
                {
                    "tier": "high",
                    "detected_on": "2026-04-15T00:00:00Z",
                    "confirmed_on": None,
                },
            ],
        )
        order = [s["uuid"] for s in tracking.list_all()]
        assert order[0] == "hot"
        assert order[-1] == "idle"
