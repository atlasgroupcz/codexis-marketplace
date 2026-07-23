"""Insolvency subject tracking — state management on top of the ISIR lookup.

A tracked *subject* is either a company (by IČO) or a natural person (name +
date of birth). Each subject may have zero or more insolvency proceedings
(řízení) in ISIR. State is stored one JSON file per subject under
$CODEXIS_PUBLIC_USER_HOME/.cdx/apps/insolvence/subjekty/<uuid>/state.json.
"""

import datetime
import json
import os
import re
import shutil
import subprocess
import unicodedata
import uuid as uuid_mod
from typing import Optional

from . import api_client
from .exceptions import (
    InsolvenceError,
    InvalidSubjectError,
    SubjectAlreadyTrackedError,
    SubjectNotTrackedError,
)
from .fs import atomic_write_json

_USER_HOME = os.environ.get("CODEXIS_PUBLIC_USER_HOME") or os.path.expanduser("~")
APP_DIR = os.path.join(_USER_HOME, ".cdx", "apps", "insolvence", "subjekty")
CDXCTL_BIN = "cdxctl"
AUTOMATION_TITLE = "Hlídač insolvencí"
AUTOMATION_CRON = "0 7 * * *"
AUTOMATION_COMMAND = "insolvence-cli sledovani check"

DRUH_LABELS = {
    "konkurs": "Konkurs",
    "oddluzeni": "Oddlužení",
    "reorganizace": "Reorganizace",
    "moratorium": "Moratorium",
    "neurceno": "Insolvenční řízení",
}


# ── time ───────────────────────────────────────────────────────────────────


def now_utc() -> str:
    return (
        datetime.datetime.now(datetime.timezone.utc)
        .isoformat(timespec="seconds")
        .replace("+00:00", "Z")
    )


# ── subject input validation ─────────────────────────────────────────────────


def normalize_ico(ico: str) -> str:
    """Strip non-digits, left-pad to 8, validate the official mod-11 checksum.

    Raises InvalidSubjectError on malformed input.
    """
    digits = re.sub(r"\D", "", ico or "")
    if not digits or len(digits) > 8:
        raise InvalidSubjectError(
            f"Neplatné IČO: {ico!r}. Očekává se 8 číslic (např. 27182775)."
        )
    digits = digits.zfill(8)
    weighted = sum(int(digits[i]) * (8 - i) for i in range(7))
    mod = weighted % 11
    check = 1 if mod == 0 else (0 if mod == 1 else 11 - mod)
    if check != int(digits[7]):
        raise InvalidSubjectError(
            f"Neplatné IČO {digits}: nesedí kontrolní číslice."
        )
    return digits


def normalize_birthdate(datum_narozeni: str) -> str:
    """Validate and canonicalise a date of birth to ISO YYYY-MM-DD."""
    value = (datum_narozeni or "").strip()
    if not value:
        raise InvalidSubjectError("Datum narození je povinné u fyzické osoby.")
    for fmt in ("%Y-%m-%d", "%d.%m.%Y", "%d. %m. %Y"):
        try:
            return datetime.datetime.strptime(value, fmt).date().isoformat()
        except ValueError:
            continue
    raise InvalidSubjectError(
        f"Neplatné datum narození: {datum_narozeni!r}. Formát RRRR-MM-DD (např. 1980-05-01)."
    )


def normalize_prijmeni(prijmeni: str) -> str:
    value = re.sub(r"\s+", " ", (prijmeni or "").strip())
    if len(value) < 2:
        raise InvalidSubjectError("Příjmení fyzické osoby je povinné.")
    return value


def normalize_jmeno(jmeno: str) -> str:
    return re.sub(r"\s+", " ", (jmeno or "").strip())


def _fold(value: str) -> str:
    """Accent- and case-insensitive folding for natural-person de-duplication."""
    folded = unicodedata.normalize("NFKD", value or "")
    folded = "".join(c for c in folded if not unicodedata.combining(c))
    return re.sub(r"\s+", " ", folded.lower().strip())


# ── state file I/O ───────────────────────────────────────────────────────────


def _state_path(subject_uuid: str) -> str:
    return os.path.join(APP_DIR, subject_uuid, "state.json")


def _load_state_dir(dir_name: str) -> Optional[dict]:
    path = os.path.join(APP_DIR, dir_name, "state.json")
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else None
    except (json.JSONDecodeError, OSError):
        return None


def _save_state(state: dict) -> None:
    atomic_write_json(_state_path(state["uuid"]), state)


def list_all() -> list:
    """Return all tracked subject state dicts. Subjects needing attention first."""
    if not os.path.isdir(APP_DIR):
        return []
    results = []
    for name in os.listdir(APP_DIR):
        state = _load_state_dir(name)
        if state is not None:
            results.append(state)

    def _sort_key(s):
        changes = s.get("changes", [])
        unconfirmed = [c for c in changes if not c.get("confirmed_on")]
        latest_unconfirmed = max(
            (c.get("detected_on", "") for c in unconfirmed), default=""
        )
        latest_change = max((c.get("detected_on", "") for c in changes), default="")
        # tier 2: unconfirmed change; tier 1: active insolvency; tier 0: idle.
        if unconfirmed:
            tier = 2
            activity = latest_unconfirmed
        elif s.get("ma_aktivni_insolvenci"):
            tier = 1
            activity = max(latest_change, s.get("added_on", ""))
        else:
            tier = 0
            activity = max(latest_change, s.get("added_on", ""))
        return (tier, activity)

    results.sort(key=_sort_key, reverse=True)
    return results


def find_by_uuid(target_uuid: str) -> Optional[dict]:
    if not target_uuid or not os.path.isdir(APP_DIR):
        return None
    state = _load_state_dir(target_uuid)
    if state and state.get("uuid") == target_uuid:
        return state
    # Fallback: scan (covers legacy dir naming).
    for name in os.listdir(APP_DIR):
        s = _load_state_dir(name)
        if s and s.get("uuid") == target_uuid:
            return s
    return None


def _find_existing_company(ico: str) -> Optional[dict]:
    for s in list_all():
        if s.get("kind") == "company" and s.get("ico") == ico:
            return s
    return None


def _find_existing_person(
    prijmeni_key: str, jmeno_key: str, datum_narozeni: str
) -> Optional[dict]:
    for s in list_all():
        if (
            s.get("kind") == "person"
            and _fold(s.get("prijmeni", "")) == prijmeni_key
            and _fold(s.get("jmeno", "")) == jmeno_key
            and s.get("datum_narozeni") == datum_narozeni
        ):
            return s
    return None


# ── change detection ─────────────────────────────────────────────────────────


def _event_key(ev: dict) -> tuple:
    return (ev.get("id") or "", ev.get("datum", ""), ev.get("popis", ""))


def _proceeding_changes(old_r: dict, new_r: dict) -> list:
    """Diff one matched proceeding (same spisová značka). Returns change records."""
    out = []
    sp = new_r.get("spisova_znacka", "")
    old_stav = old_r.get("stav", "")
    new_stav = new_r.get("stav", "")
    if new_stav and new_stav != old_stav:
        out.append(
            {
                "detected_on": now_utc(),
                "typ": "zmena_stavu",
                "spisova_znacka": sp,
                "popis": f"Změna stavu řízení {sp}: {old_stav or '—'} → {new_stav}",
                "old_stav": old_stav,
                "new_stav": new_stav,
                "confirmed_on": None,
            }
        )
    old_events = {_event_key(e) for e in old_r.get("udalosti", [])}
    for ev in new_r.get("udalosti", []):
        if _event_key(ev) not in old_events:
            out.append(
                {
                    "detected_on": now_utc(),
                    "typ": "nova_udalost",
                    "spisova_znacka": sp,
                    "popis": f"Nová událost v řízení {sp}: {ev.get('popis', '')}".strip(),
                    "udalost": ev,
                    "confirmed_on": None,
                }
            )
    return out


def detect_changes(old_rizeni: list, new_rizeni: list) -> list:
    """Compare stored proceedings vs fresh ISIR data, return new change records."""
    out = []
    old_by_sp = {r.get("spisova_znacka", ""): r for r in old_rizeni}
    for new_r in new_rizeni:
        sp = new_r.get("spisova_znacka", "")
        old_r = old_by_sp.get(sp)
        if old_r is None:
            out.append(
                {
                    "detected_on": now_utc(),
                    "typ": "nove_rizeni",
                    "spisova_znacka": sp,
                    "popis": (
                        f"Nové insolvenční řízení {sp} "
                        f"({DRUH_LABELS.get(new_r.get('druh', 'neurceno'), 'Insolvenční řízení')})"
                    ),
                    "druh": new_r.get("druh", "neurceno"),
                    "new_stav": new_r.get("stav", ""),
                    "confirmed_on": None,
                }
            )
        else:
            out.extend(_proceeding_changes(old_r, new_r))
    return out


# ── automation ───────────────────────────────────────────────────────────────


def ensure_automation() -> None:
    """Ensure the central daily cron automation exists. Idempotent, non-fatal."""
    try:
        result = subprocess.run(
            [CDXCTL_BIN, "automation", "list"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        if result.returncode != 0:
            return
        automations = json.loads(result.stdout)
    except Exception:
        return

    if not isinstance(automations, list):
        return
    for a in automations:
        if (
            isinstance(a, dict)
            and a.get("type") == "COMMAND"
            and a.get("command") == AUTOMATION_COMMAND
        ):
            return

    try:
        subprocess.run(
            [
                CDXCTL_BIN,
                "automation",
                "create-command",
                "--title",
                AUTOMATION_TITLE,
                "--cron",
                AUTOMATION_CRON,
                "--command",
                AUTOMATION_COMMAND,
                "--description",
                "Pravidelná kontrola sledovaných subjektů v insolvenčním rejstříku (ISIR).",
            ],
            capture_output=True,
            text=True,
            timeout=15,
        )
    except Exception:
        return


# ── high-level operations ────────────────────────────────────────────────────


def _new_subject_skeleton(kind: str, label: str) -> dict:
    return {
        "uuid": str(uuid_mod.uuid4()),
        "kind": kind,
        "label": label or "",
        "added_on": now_utc(),
        "last_check_at": now_utc(),
        "ma_aktivni_insolvenci": False,
        "rizeni": [],
        "changes": [],
    }


def add_company(ico: str, label: str = "") -> dict:
    """Start tracking a company by IČO. Fetches the current ISIR state as baseline."""
    canonical = normalize_ico(ico)
    if _find_existing_company(canonical) is not None:
        raise SubjectAlreadyTrackedError(f"Firma s IČO {canonical} je již sledována.")

    lookup = api_client.lookup_company(canonical)
    state = _new_subject_skeleton("company", label)
    state["ico"] = canonical
    state["nazev"] = lookup.get("nazev") or ""
    state["rizeni"] = lookup.get("rizeni", [])
    state["ma_aktivni_insolvenci"] = any(r.get("aktivni") for r in state["rizeni"])
    _save_state(state)
    ensure_automation()
    return state


def add_person(prijmeni: str, jmeno: str, datum_narozeni: str, label: str = "") -> dict:
    """Start tracking a natural person by surname + given name + date of birth."""
    surname = normalize_prijmeni(prijmeni)
    given = normalize_jmeno(jmeno)
    dob = normalize_birthdate(datum_narozeni)
    if _find_existing_person(_fold(surname), _fold(given), dob) is not None:
        raise SubjectAlreadyTrackedError(
            f"Osoba {given} {surname} ({dob}) je již sledována."
        )

    lookup = api_client.lookup_person(surname, given, dob)
    state = _new_subject_skeleton("person", label)
    state["prijmeni"] = surname
    state["jmeno"] = given
    state["datum_narozeni"] = dob
    state["rizeni"] = lookup.get("rizeni", [])
    state["ma_aktivni_insolvenci"] = any(r.get("aktivni") for r in state["rizeni"])
    _save_state(state)
    ensure_automation()
    return state


def set_label(subject_uuid: str, label: str) -> dict:
    state = find_by_uuid(subject_uuid)
    if state is None:
        raise SubjectNotTrackedError("Subjekt není sledován.")
    state["label"] = label or ""
    _save_state(state)
    return state


def remove(subject_uuid: str) -> None:
    """Stop tracking a subject. Rename-then-rmtree keeps a crash recoverable."""
    state = find_by_uuid(subject_uuid)
    if state is None:
        raise SubjectNotTrackedError("Subjekt není sledován.")
    target = os.path.join(APP_DIR, state["uuid"])
    if os.path.isdir(target):
        trash = f"{target}.deleted-{uuid_mod.uuid4().hex}"
        os.rename(target, trash)
        shutil.rmtree(trash, ignore_errors=True)


def show(subject_uuid: str) -> dict:
    state = find_by_uuid(subject_uuid)
    if state is None:
        raise SubjectNotTrackedError("Subjekt není sledován.")
    return state


def _lookup_for(state: dict) -> dict:
    if state.get("kind") == "company":
        return api_client.lookup_company(state["ico"])
    return api_client.lookup_person(
        state.get("prijmeni", ""), state.get("jmeno", ""), state["datum_narozeni"]
    )


def check_one(subject_uuid: str) -> dict:
    """Check a single tracked subject for changes against ISIR."""
    state = find_by_uuid(subject_uuid)
    if state is None:
        raise SubjectNotTrackedError("Subjekt není sledován.")
    try:
        lookup = _lookup_for(state)
    except InsolvenceError as e:
        return {
            "uuid": subject_uuid,
            "display_name": display_name(state),
            "ok": False,
            "changes": [],
            "error": str(e),
        }

    new_rizeni = lookup.get("rizeni", [])
    changes = detect_changes(state.get("rizeni", []), new_rizeni)

    state["last_check_at"] = now_utc()
    state["rizeni"] = new_rizeni
    state["ma_aktivni_insolvenci"] = any(r.get("aktivni") for r in new_rizeni)
    if state.get("kind") == "company" and lookup.get("nazev"):
        state["nazev"] = lookup["nazev"]
    if changes:
        state.setdefault("changes", []).extend(changes)
    _save_state(state)

    return {
        "uuid": subject_uuid,
        "display_name": display_name(state),
        "ok": True,
        "changes": changes,
        "error": None,
    }


def check_all() -> list:
    results = []
    for state in list_all():
        try:
            results.append(check_one(state["uuid"]))
        except InsolvenceError as e:
            results.append(
                {
                    "uuid": state.get("uuid", "?"),
                    "display_name": display_name(state),
                    "ok": False,
                    "changes": [],
                    "error": str(e),
                }
            )
    return results


def confirm(subject_uuid: str, change_index: Optional[int] = None) -> int:
    """Mark changes as read. Returns number of changes marked."""
    state = find_by_uuid(subject_uuid)
    if state is None:
        raise SubjectNotTrackedError("Subjekt není sledován.")
    changes = state.get("changes", [])
    timestamp = now_utc()
    marked = 0
    if change_index is not None:
        if not (0 <= change_index < len(changes)):
            raise IndexError(
                f"Index {change_index} je mimo rozsah (0..{len(changes) - 1})."
            )
        if not changes[change_index].get("confirmed_on"):
            changes[change_index]["confirmed_on"] = timestamp
            marked = 1
    else:
        for c in changes:
            if not c.get("confirmed_on"):
                c["confirmed_on"] = timestamp
                marked += 1
    if marked > 0:
        _save_state(state)
    return marked


# ── presentation helpers ─────────────────────────────────────────────────────


def display_name(state: dict) -> str:
    if state.get("label"):
        return state["label"]
    if state.get("kind") == "company":
        return state.get("nazev") or f"IČO {state.get('ico', '')}"
    person = f"{state.get('jmeno', '')} {state.get('prijmeni', '')}".strip()
    return person or "?"


def _primary_proceeding(rizeni: list) -> Optional[dict]:
    """The proceeding to surface in the overview (active first, then most recent)."""
    if not rizeni:
        return None
    return sorted(
        rizeni,
        key=lambda r: (bool(r.get("aktivni")), r.get("datum_zahajeni", "")),
        reverse=True,
    )[0]


def state_to_overview_entry(state: dict) -> dict:
    rizeni = state.get("rizeni", [])
    primary = _primary_proceeding(rizeni)
    changes = state.get("changes", [])
    unconfirmed = sum(1 for c in changes if not c.get("confirmed_on"))
    return {
        "uuid": state.get("uuid", ""),
        "kind": state.get("kind", ""),
        "label": state.get("label", ""),
        "display_name": display_name(state),
        "ico": state.get("ico", ""),
        "nazev": state.get("nazev", ""),
        "prijmeni": state.get("prijmeni", ""),
        "jmeno": state.get("jmeno", ""),
        "datum_narozeni": state.get("datum_narozeni", ""),
        "ma_aktivni_insolvenci": bool(state.get("ma_aktivni_insolvenci")),
        "rizeni_count": len(rizeni),
        "primary_druh": primary.get("druh") if primary else None,
        "primary_druh_label": (
            DRUH_LABELS.get(primary.get("druh", "neurceno"), "Insolvenční řízení")
            if primary
            else None
        ),
        "primary_stav": primary.get("stav") if primary else None,
        "added_on": state.get("added_on", ""),
        "last_check_at": state.get("last_check_at", ""),
        "changes_count": len(changes),
        "unconfirmed_count": unconfirmed,
    }


def state_to_detail_entry(state: dict) -> dict:
    entry = state_to_overview_entry(state)
    entry["rizeni"] = state.get("rizeni", [])
    entry["changes"] = state.get("changes", [])
    return entry
