"""Cadastral proceeding tracking — state management on top of ČÚZK API."""

import datetime
import json
import os
import re
import shutil
import subprocess
import urllib.error
import uuid as uuid_mod
from typing import Optional

from . import api_client
from .exceptions import (
    InvalidProceedingNumberError,
    KatastrError,
    ProceedingAlreadyTrackedError,
    ProceedingNotFoundError,
    ProceedingNotTrackedError,
)
from .fs import atomic_write_json

_USER_HOME = os.environ.get("CDX_USER_HOME") or os.path.expanduser("~")
APP_DIR = os.path.join(_USER_HOME, ".cdx", "apps", "katastr", "rizeni")
CDXCTL_BIN = "cdxctl"
AUTOMATION_TITLE = "Hlídač katastrálních řízení"
AUTOMATION_CRON = "0 8 * * 1"
AUTOMATION_COMMAND = "katastr-cli tracking check"

# TYPE-NUMBER/YEAR-WORKPLACE, e.g. V-123/2026-701
RIZENI_PATTERN = re.compile(
    r"^(?P<typ>[A-Z]+)-(?P<cislo>\d+)/(?P<rok>\d{4})-(?P<pracoviste>\d+)$",
    re.IGNORECASE,
)

UHRADA_LABELS = {
    "U": "Uhrazeno",
    "N": "Neuhrazeno",
    "O": "Osvobozeno",
}


# ── helpers ──────────────────────────────────────────────────────────────────


def now_utc() -> str:
    return (
        datetime.datetime.now(datetime.timezone.utc)
        .isoformat(timespec="seconds")
        .replace("+00:00", "Z")
    )


def parse_proceeding_number(cislo_rizeni: str) -> dict:
    """Parse e.g. V-123/2026-701 into a dict.

    Raises:
        InvalidProceedingNumberError: when format doesn't match.
    """
    if not cislo_rizeni:
        raise InvalidProceedingNumberError("Prázdné číslo řízení.")
    m = RIZENI_PATTERN.match(cislo_rizeni.strip())
    if not m:
        raise InvalidProceedingNumberError(
            f"Neplatný formát čísla řízení: {cislo_rizeni}. "
            "Očekávaný formát: TYP-ČÍSLO/ROK-KÓD (např. V-123/2026-701)."
        )
    return {
        "typ_rizeni": m.group("typ").upper(),
        "poradove_cislo": int(m.group("cislo")),
        "rok": int(m.group("rok")),
        "kod_pracoviste": int(m.group("pracoviste")),
    }


def format_proceeding_number(parsed: dict) -> str:
    return (
        f"{parsed['typ_rizeni']}-{parsed['poradove_cislo']}"
        f"/{parsed['rok']}-{parsed['kod_pracoviste']}"
    )


def normalize_number(cislo_rizeni: str) -> str:
    """Parse + format to canonical form (uppercase type, no whitespace)."""
    return format_proceeding_number(parse_proceeding_number(cislo_rizeni))


def _dir_name(cislo_rizeni: str) -> str:
    """Filesystem-safe directory name (slash → underscore)."""
    return cislo_rizeni.replace("/", "_")


def _state_path(cislo_rizeni: str) -> str:
    return os.path.join(APP_DIR, _dir_name(cislo_rizeni), "state.json")


def _load_state_file(dir_name: str) -> Optional[dict]:
    """Load state.json from a given directory name. Returns None if missing/broken."""
    path = os.path.join(APP_DIR, dir_name, "state.json")
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else None
    except (json.JSONDecodeError, OSError):
        return None


def _save_state(cislo_rizeni: str, state: dict) -> None:
    """Atomically write state.json (tmp file + rename)."""
    atomic_write_json(_state_path(cislo_rizeni), state)


def _load_state(cislo_rizeni: str) -> Optional[dict]:
    return _load_state_file(_dir_name(cislo_rizeni))


# ── ČÚZK fetch ───────────────────────────────────────────────────────────────


def fetch_from_cuzk(parsed: dict) -> Optional[dict]:
    """Fetch proceeding data from ČÚZK. Returns the data dict or None if not found."""
    path = (
        f"/api/v1/Rizeni/Vyhledani?"
        f"TypRizeni={parsed['typ_rizeni']}"
        f"&Cislo={parsed['poradove_cislo']}"
        f"&Rok={parsed['rok']}"
        f"&KodPracoviste={parsed['kod_pracoviste']}"
    )
    result = api_client.get(path)
    data = result.get("data", [])
    return data[0] if data else None


# ── change detection ─────────────────────────────────────────────────────────


def _detect_changes(old_state: dict, new_data: dict) -> Optional[dict]:
    """Compare old state vs fresh API data, return change record or None."""
    old_stav = old_state.get("stav", "")
    new_stav = new_data.get("stav", "")
    old_ops = old_state.get("provedene_operace", [])
    new_ops = new_data.get("provedeneOperace", [])
    old_uhrada = old_state.get("stav_uhrady")
    new_uhrada = new_data.get("stavUhrady")

    old_op_set = {
        (op.get("nazev", ""), op.get("datumProvedeni", "")) for op in old_ops
    }
    new_operations = [
        op
        for op in new_ops
        if (op.get("nazev", ""), op.get("datumProvedeni", "")) not in old_op_set
    ]

    if (
        new_stav == old_stav
        and not new_operations
        and new_uhrada == old_uhrada
    ):
        return None

    return {
        "detected_on": now_utc(),
        "old_stav": old_stav,
        "new_stav": new_stav,
        "new_operations": new_operations,
        "stav_uhrady_changed": new_uhrada != old_uhrada,
        "old_stav_uhrady": old_uhrada,
        "new_stav_uhrady": new_uhrada,
        "confirmed_on": None,
    }


# ── automation ───────────────────────────────────────────────────────────────


def ensure_automation() -> None:
    """Ensure the central cron automation exists. Idempotent, non-fatal on failure."""
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
                "Pravidelná kontrola sledovaných katastrálních řízení.",
            ],
            capture_output=True,
            text=True,
            timeout=15,
        )
    except Exception:
        return


# ── high-level operations ────────────────────────────────────────────────────


def add(cislo_rizeni: str, label: str = "") -> dict:
    """Start tracking a proceeding. Fetches current state from ČÚZK as baseline.

    Returns the saved state dict.

    Raises:
        InvalidProceedingNumberError: bad format
        ProceedingAlreadyTrackedError: already tracked
        ProceedingNotFoundError: doesn't exist in ČÚZK
        ApiKeyMissingError / ApiKeyInvalidError / ApiHttpError / ApiNetworkError
    """
    parsed = parse_proceeding_number(cislo_rizeni)
    canonical = format_proceeding_number(parsed)

    if _load_state(canonical) is not None:
        raise ProceedingAlreadyTrackedError(
            f"Řízení {canonical} je již sledováno."
        )

    data = fetch_from_cuzk(parsed)
    if data is None:
        raise ProceedingNotFoundError(
            f"Řízení {canonical} nebylo nalezeno v ČÚZK."
        )

    state = {
        "uuid": str(uuid_mod.uuid4()),
        "cislo_rizeni": canonical,
        "typ_rizeni": parsed["typ_rizeni"],
        "poradove_cislo": parsed["poradove_cislo"],
        "rok": parsed["rok"],
        "kod_pracoviste": parsed["kod_pracoviste"],
        "api_id": data.get("id"),
        "label": label or "",
        "added_on": now_utc(),
        "last_check_at": now_utc(),
        "stav": data.get("stav", ""),
        "stav_uhrady": data.get("stavUhrady"),
        "datum_prijeti": data.get("datumPrijeti", ""),
        "provedene_operace": data.get("provedeneOperace", []),
        "changes": [],
    }
    _save_state(canonical, state)
    ensure_automation()
    return state


def set_label(cislo_rizeni: str, label: str) -> dict:
    """Update the user-defined label of a tracked proceeding.

    Raises ProceedingNotTrackedError if not tracked.
    Returns the updated state dict.
    """
    canonical = normalize_number(cislo_rizeni)
    state = _load_state(canonical)
    if state is None:
        raise ProceedingNotTrackedError(
            f"Řízení {canonical} není sledováno."
        )
    state["label"] = label or ""
    _save_state(canonical, state)
    return state


def remove(cislo_rizeni: str) -> None:
    """Stop tracking a proceeding. Rename-then-rmtree keeps a crash recoverable."""
    canonical = normalize_number(cislo_rizeni)
    if _load_state(canonical) is None:
        raise ProceedingNotTrackedError(
            f"Řízení {canonical} není sledováno."
        )
    target = os.path.join(APP_DIR, _dir_name(canonical))
    trash = f"{target}.deleted-{uuid_mod.uuid4().hex}"
    os.rename(target, trash)
    shutil.rmtree(trash, ignore_errors=True)


def show(cislo_rizeni: str) -> dict:
    """Return full state of a tracked proceeding.

    Raises ProceedingNotTrackedError if not tracked.
    """
    canonical = normalize_number(cislo_rizeni)
    state = _load_state(canonical)
    if state is None:
        raise ProceedingNotTrackedError(
            f"Řízení {canonical} není sledováno. "
            "Použij `katastr tracking add` pro přidání nebo "
            "`katastr api get` pro one-shot lookup."
        )
    return state


def list_all() -> list:
    """Return list of all tracked proceeding state dicts, newest first."""
    if not os.path.isdir(APP_DIR):
        return []
    results = []
    for name in os.listdir(APP_DIR):
        state = _load_state_file(name)
        if state is not None:
            results.append(state)
    def _sort_key(s):
        changes = s.get("changes", [])
        unconfirmed = [c for c in changes if not c.get("confirmed_on")]
        latest_unconfirmed = max(
            (c.get("detected_on", "") for c in unconfirmed), default=""
        )
        latest_confirmed = max(
            (c.get("confirmed_on", "") for c in changes if c.get("confirmed_on")),
            default="",
        )
        # tier: 2 = unconfirmed, 1 = has changes, 0 = no changes
        tier = 2 if unconfirmed else (1 if changes else 0)
        activity = latest_unconfirmed if unconfirmed else latest_confirmed
        return (tier, activity, s.get("added_on", ""))

    results.sort(key=_sort_key, reverse=True)
    return results


def find_by_uuid(target_uuid: str) -> Optional[dict]:
    """Find a tracked proceeding by uuid."""
    if not os.path.isdir(APP_DIR):
        return None
    for name in os.listdir(APP_DIR):
        state = _load_state_file(name)
        if state and state.get("uuid") == target_uuid:
            return state
    return None


def check_one(cislo_rizeni: str) -> dict:
    """Check a single tracked proceeding for changes. Returns:
    {
        "cislo_rizeni": str,
        "ok": bool,
        "change": dict | None,
        "error": str | None,
    }
    """
    canonical = normalize_number(cislo_rizeni)
    state = _load_state(canonical)
    if state is None:
        raise ProceedingNotTrackedError(
            f"Řízení {canonical} není sledováno."
        )
    parsed = parse_proceeding_number(canonical)
    try:
        data = fetch_from_cuzk(parsed)
    except (KatastrError, urllib.error.URLError) as e:
        return {
            "cislo_rizeni": canonical,
            "ok": False,
            "change": None,
            "error": str(e),
        }
    if data is None:
        return {
            "cislo_rizeni": canonical,
            "ok": False,
            "change": None,
            "error": "Řízení nebylo nalezeno v ČÚZK API.",
        }

    change = _detect_changes(state, data)

    state["last_check_at"] = now_utc()
    state["stav"] = data.get("stav", state.get("stav", ""))
    state["stav_uhrady"] = data.get("stavUhrady")
    state["provedene_operace"] = data.get("provedeneOperace", [])
    if change:
        state.setdefault("changes", []).append(change)
    _save_state(canonical, state)

    return {
        "cislo_rizeni": canonical,
        "ok": True,
        "change": change,
        "error": None,
    }


def check_all() -> list:
    """Check all tracked proceedings. Returns list of per-proceeding results."""
    results = []
    for state in list_all():
        try:
            results.append(check_one(state["cislo_rizeni"]))
        except (KatastrError, urllib.error.URLError) as e:
            results.append(
                {
                    "cislo_rizeni": state.get("cislo_rizeni", "?"),
                    "ok": False,
                    "change": None,
                    "error": str(e),
                }
            )
    return results


def confirm(cislo_rizeni: str, change_index: Optional[int] = None) -> int:
    """Mark changes as read. Returns number of changes marked.

    Args:
        cislo_rizeni: tracked proceeding number
        change_index: if given, mark only that specific change.
                      If None (default), mark all unconfirmed changes.
    """
    canonical = normalize_number(cislo_rizeni)
    state = _load_state(canonical)
    if state is None:
        raise ProceedingNotTrackedError(
            f"Řízení {canonical} není sledováno."
        )

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
        _save_state(canonical, state)
    return marked


# ── presentation helpers ─────────────────────────────────────────────────────


def uhrada_label(stav_uhrady: Optional[str]) -> Optional[str]:
    """Map ČÚZK payment status to display label."""
    if stav_uhrady is None:
        return None
    return UHRADA_LABELS.get(stav_uhrady)


def state_to_overview_entry(state: dict) -> dict:
    """Convert internal state to a frontend-friendly overview entry."""
    ops = state.get("provedene_operace", [])
    last_op_date = None
    if ops:
        dates = [op.get("datumProvedeni", "") for op in ops if op.get("datumProvedeni")]
        if dates:
            last_op_date = max(dates)

    changes = state.get("changes", [])
    unconfirmed = sum(1 for c in changes if not c.get("confirmed_on"))

    return {
        "uuid": state.get("uuid", ""),
        "cislo_rizeni": state.get("cislo_rizeni", ""),
        "typ_rizeni": state.get("typ_rizeni", ""),
        "label": state.get("label", ""),
        "stav": state.get("stav", ""),
        "stav_uhrady": state.get("stav_uhrady"),
        "stav_uhrady_label": uhrada_label(state.get("stav_uhrady")),
        "datum_prijeti": state.get("datum_prijeti", ""),
        "added_on": state.get("added_on", ""),
        "last_check_at": state.get("last_check_at", ""),
        "last_op_date": last_op_date,
        "operace_count": len(ops),
        "changes_count": len(changes),
        "unconfirmed_count": unconfirmed,
        "provedene_operace": [
            {
                "nazev": op.get("nazev", ""),
                "datumProvedeni": op.get("datumProvedeni", ""),
            }
            for op in ops
        ],
    }
