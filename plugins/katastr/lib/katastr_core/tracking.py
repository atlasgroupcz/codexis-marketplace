"""Cadastral proceeding tracking — state management on top of ČÚZK API."""

import datetime
import json
import os
import re
import shutil
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
from .fs import atomic_write_json, app_path

APP_DIR = app_path("rizeni")

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


def is_unconfirmed(change: dict) -> bool:
    """A change stays unconfirmed (unread) until it has a confirmed_on timestamp."""
    return not change.get("confirmed_on")


def _dir_name(cislo_rizeni: str) -> str:
    """Filesystem-safe directory name (slash → underscore)."""
    return cislo_rizeni.replace("/", "_")


def _state_path_for_dir(dir_name: str) -> str:
    return os.path.join(APP_DIR, dir_name, "state.json")


def _state_path(cislo_rizeni: str) -> str:
    return _state_path_for_dir(_dir_name(cislo_rizeni))


def _load_state_file(dir_name: str) -> Optional[dict]:
    """Load state.json from a given directory name. Returns None if missing/broken."""
    path = _state_path_for_dir(dir_name)
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


def fetch_by_id(api_id) -> Optional[dict]:
    """Fetch a proceeding directly by its ČÚZK id. Returns the data dict or None.

    Unlike `Rizeni/Vyhledani` (whose `data` is a list), `Rizeni/{id}` wraps a
    single object under `data` — verified live against api-kn.cuzk.gov.cz
    2026-07: same envelope key, same field names (`stav`, `stavUhrady`,
    `provedeneOperace`, `datumPrijeti`, `id`), no `[0]` unwrap needed.
    """
    result = api_client.get(f"/api/v1/Rizeni/{api_id}")
    data = result.get("data")
    return data or None


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
    """Return list of all tracked proceeding state dicts, newest added first."""
    if not os.path.isdir(APP_DIR):
        return []
    results = []
    for name in os.listdir(APP_DIR):
        state = _load_state_file(name)
        if state is not None:
            results.append(state)
    # Newest first, purely by date added. Confirming a change — or a newly
    # detected one — never reorders; order is stable once a proceeding is
    # added. uuid is a deterministic tiebreaker when two share an added_on
    # second.
    results.sort(
        key=lambda s: (s.get("added_on", ""), s.get("uuid", "")),
        reverse=True,
    )
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


def _check_result(cislo_rizeni, ok, change=None, error=None) -> dict:
    """Build the uniform per-proceeding check result dict."""
    return {"cislo_rizeni": cislo_rizeni, "ok": ok, "change": change, "error": error}


def _check_state(state: dict) -> dict:
    """Check an already-loaded state dict against ČÚZK. Persists + returns result.

    `state` carries the parsed number parts (typ_rizeni/poradove_cislo/rok/
    kod_pracoviste), so no re-parse is needed and callers that already hold the
    state (check_all) avoid a second read of state.json.
    """
    canonical = state["cislo_rizeni"]
    try:
        api_id = state.get("api_id")
        data = fetch_by_id(api_id) if api_id else fetch_from_cuzk(state)
    except (KatastrError, urllib.error.URLError) as e:
        return _check_result(canonical, False, error=str(e))
    if data is None:
        return _check_result(
            canonical, False, error="Řízení nebylo nalezeno v ČÚZK API."
        )

    change = _detect_changes(state, data)

    state["last_check_at"] = now_utc()
    # Refresh the ČÚZK-sourced fields, same shape as add(). api_id and
    # datum_prijeti are backfilled but never wiped: keep the stored value when
    # ČÚZK omits it (datum_prijeti is immutable; a missing id would else strand
    # us on the slower Vyhledani path forever).
    state["api_id"] = data.get("id") or state.get("api_id")
    state["stav"] = data.get("stav", state.get("stav", ""))
    state["stav_uhrady"] = data.get("stavUhrady")
    state["datum_prijeti"] = data.get("datumPrijeti") or state.get("datum_prijeti", "")
    state["provedene_operace"] = data.get("provedeneOperace", [])
    if change:
        state.setdefault("changes", []).append(change)
    _save_state(canonical, state)

    return _check_result(canonical, True, change=change)


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
    return _check_state(state)


def check_all() -> list:
    """Check all tracked proceedings. Returns list of per-proceeding results."""
    results = []
    for state in list_all():
        try:
            results.append(_check_state(state))
        except (KatastrError, urllib.error.URLError) as e:
            results.append(
                _check_result(state.get("cislo_rizeni", "?"), False, error=str(e))
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


def _uhrada_display(stav_uhrady: Optional[str]) -> str:
    return uhrada_label(stav_uhrady) or stav_uhrady or "—"


def describe_change(change: dict) -> list:
    """Human-readable, unindented lines describing one detected change.

    Single source shared by the CLI check output and the e-mail summary, so a
    change (stav / new operations / úhrada) renders the same in both channels.
    Each caller adds its own prefix/indentation.
    """
    lines = []
    if change.get("old_stav") != change.get("new_stav"):
        lines.append(f"Stav: {change.get('old_stav', '')} → {change.get('new_stav', '')}")
    for op in change.get("new_operations", []):
        lines.append(
            f"Nová operace: {op.get('nazev', '?')} ({op.get('datumProvedeni', '')})"
        )
    if change.get("stav_uhrady_changed"):
        lines.append(
            f"Úhrada: {_uhrada_display(change.get('old_stav_uhrady'))} "
            f"→ {_uhrada_display(change.get('new_stav_uhrady'))}"
        )
    return lines


def _ordered_operace(state: dict) -> list:
    """Operations oldest → newest. ČÚZK's `provedeneOperace` is already in
    chronological (oldest→newest) order — verified live 2026-07-01 across 9
    proceedings, strictly ascending by datumProvedeni. So pass it through as-is,
    do NOT reverse. Operations sharing an identical timestamp (e.g. Založení /
    Zaplombování at intake) keep ČÚZK's own sub-order — we have no signal to
    reorder them and ČÚZK itself is inconsistent there."""
    return list(state.get("provedene_operace", []))


def state_to_overview_entry(state: dict) -> dict:
    """Convert internal state to a frontend-friendly overview entry."""
    ops = _ordered_operace(state)
    last_op_date = ops[-1].get("datumProvedeni") if ops else None

    changes = state.get("changes", [])
    unconfirmed = sum(1 for c in changes if is_unconfirmed(c))

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
