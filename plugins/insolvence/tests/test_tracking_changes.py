"""Change detection, overview entry, and list_all sort order."""

import json
import os

from insolvence_core.tracking import (
    detect_changes,
    list_all,
    state_to_overview_entry,
)


def _rizeni(sp, stav="Konkurs", druh="konkurs", aktivni=True, udalosti=None, datum="2026-01-01"):
    return {
        "spisova_znacka": sp,
        "soud": "Městský soud v Praze",
        "druh": druh,
        "stav": stav,
        "datum_zahajeni": datum,
        "datum_ukonceni": "" if aktivni else "2026-05-01",
        "aktivni": aktivni,
        "udalosti": udalosti or [],
    }


def _seed_subject(app_dir, uuid, **overrides):
    base = {
        "uuid": uuid,
        "kind": "company",
        "label": "",
        "ico": "00000000",
        "nazev": f"Firma {uuid}",
        "added_on": "2026-04-01T00:00:00Z",
        "last_check_at": "2026-04-01T00:00:00Z",
        "ma_aktivni_insolvenci": False,
        "rizeni": [],
        "changes": [],
    }
    base.update(overrides)
    dpath = os.path.join(str(app_dir), uuid)
    os.makedirs(dpath, exist_ok=True)
    with open(os.path.join(dpath, "state.json"), "w", encoding="utf-8") as f:
        json.dump(base, f)
    return base


# ── detect_changes ───────────────────────────────────────────────────────────


class TestDetectChanges:
    def test_no_change(self):
        old = [_rizeni("MSPH 78 INS 1/2026")]
        new = [_rizeni("MSPH 78 INS 1/2026")]
        assert detect_changes(old, new) == []

    def test_new_proceeding(self):
        changes = detect_changes([], [_rizeni("MSPH 78 INS 1/2026", druh="konkurs")])
        assert len(changes) == 1
        assert changes[0]["typ"] == "nove_rizeni"
        assert changes[0]["spisova_znacka"] == "MSPH 78 INS 1/2026"
        assert changes[0]["confirmed_on"] is None

    def test_stav_change(self):
        old = [_rizeni("MSPH 78 INS 1/2026", stav="Úpadek")]
        new = [_rizeni("MSPH 78 INS 1/2026", stav="Konkurs")]
        changes = detect_changes(old, new)
        assert len(changes) == 1
        assert changes[0]["typ"] == "zmena_stavu"
        assert changes[0]["old_stav"] == "Úpadek"
        assert changes[0]["new_stav"] == "Konkurs"

    def test_new_event(self):
        old = [_rizeni("MSPH 78 INS 1/2026", udalosti=[{"datum": "2026-01-02", "popis": "A"}])]
        new = [
            _rizeni(
                "MSPH 78 INS 1/2026",
                udalosti=[
                    {"datum": "2026-01-02", "popis": "A"},
                    {"datum": "2026-02-02", "popis": "B"},
                ],
            )
        ]
        changes = detect_changes(old, new)
        assert len(changes) == 1
        assert changes[0]["typ"] == "nova_udalost"
        assert changes[0]["udalost"]["popis"] == "B"

    def test_multiple_changes_in_one_check(self):
        old = [_rizeni("MSPH 78 INS 1/2026", stav="Úpadek")]
        new = [
            _rizeni("MSPH 78 INS 1/2026", stav="Konkurs"),
            _rizeni("KSBR 30 INS 9/2026", stav="Oddlužení", druh="oddluzeni"),
        ]
        changes = detect_changes(old, new)
        typy = sorted(c["typ"] for c in changes)
        assert typy == ["nove_rizeni", "zmena_stavu"]


# ── state_to_overview_entry ──────────────────────────────────────────────────


class TestOverviewEntry:
    def test_no_proceedings(self):
        entry = state_to_overview_entry(
            {"uuid": "a", "kind": "company", "ico": "00000000", "rizeni": []}
        )
        assert entry["rizeni_count"] == 0
        assert entry["primary_druh"] is None
        assert entry["primary_stav"] is None
        assert entry["unconfirmed_count"] == 0

    def test_primary_prefers_active(self):
        entry = state_to_overview_entry(
            {
                "uuid": "a",
                "kind": "company",
                "rizeni": [
                    _rizeni("OLD", druh="oddluzeni", aktivni=False, datum="2020-01-01"),
                    _rizeni("NEW", druh="konkurs", aktivni=True, datum="2019-01-01"),
                ],
            }
        )
        # active (konkurs) wins even though it started earlier
        assert entry["primary_druh"] == "konkurs"
        assert entry["primary_druh_label"] == "Konkurs"

    def test_counts_unconfirmed(self):
        entry = state_to_overview_entry(
            {
                "uuid": "a",
                "changes": [
                    {"confirmed_on": None},
                    {"confirmed_on": "2026-04-10T00:00:00Z"},
                    {"confirmed_on": None},
                ],
            }
        )
        assert entry["changes_count"] == 3
        assert entry["unconfirmed_count"] == 2

    def test_person_display_name(self):
        entry = state_to_overview_entry(
            {
                "uuid": "a",
                "kind": "person",
                "prijmeni": "Novák",
                "jmeno": "Jan",
                "datum_narozeni": "1980-05-01",
            }
        )
        assert entry["display_name"] == "Jan Novák"


# ── list_all sort order ──────────────────────────────────────────────────────


class TestListAllSort:
    def test_empty_when_dir_missing(self, app_dir):
        os.rmdir(str(app_dir))
        assert list_all() == []

    def test_unconfirmed_beats_active_beats_idle(self, app_dir):
        _seed_subject(app_dir, "idle", ma_aktivni_insolvenci=False, changes=[])
        _seed_subject(
            app_dir,
            "active",
            ma_aktivni_insolvenci=True,
            rizeni=[_rizeni("X")],
            changes=[{"detected_on": "2026-04-10T00:00:00Z", "confirmed_on": "2026-04-11T00:00:00Z"}],
        )
        _seed_subject(
            app_dir,
            "unread",
            ma_aktivni_insolvenci=True,
            changes=[{"detected_on": "2026-04-15T00:00:00Z", "confirmed_on": None}],
        )
        order = [s["uuid"] for s in list_all()]
        assert order == ["unread", "active", "idle"]

    def test_ignores_broken_state_files(self, app_dir):
        _seed_subject(app_dir, "good")
        bad = os.path.join(str(app_dir), "bad")
        os.makedirs(bad)
        with open(os.path.join(bad, "state.json"), "w") as f:
            f.write("{not valid json")
        result = list_all()
        assert len(result) == 1
        assert result[0]["uuid"] == "good"
