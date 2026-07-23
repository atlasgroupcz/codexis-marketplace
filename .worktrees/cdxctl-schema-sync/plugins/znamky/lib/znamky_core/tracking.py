"""Trademark watch-state management on top of the public source search.

A watched *mark* is the sign a user wants to protect — a word ("CODEXIS"), a
logo (figurative), or both (combined) — scoped to Nice classes and territories
(CZ / EU). For each watched mark we periodically search the public registers
(via znamky_core.sources), score every hit for similarity, and record the ones
above the mark's threshold as *collisions*. New collisions appearing on a later
check are what the watchdog alerts on.

State is one JSON file per mark under
$CODEXIS_PUBLIC_USER_HOME/.cdx/apps/znamky/sledovane/<uuid>/state.json, with any
stored logos beside it — kept on disk so AI tools can read them as context.
"""

import base64
import datetime
import json
import os
import re
import shutil
import subprocess
import uuid as uuid_mod
from typing import Optional

from . import image_similarity, scoring, sources
from .exceptions import (
    InvalidMarkError,
    MarkNotTrackedError,
    ZnamkyError,
)
from .fs import atomic_write_bytes, atomic_write_json

_USER_HOME = os.environ.get("CODEXIS_PUBLIC_USER_HOME") or os.path.expanduser("~")
APP_DIR = os.path.join(_USER_HOME, ".cdx", "apps", "znamky", "sledovane")

CDXCTL_BIN = "cdxctl"
AUTOMATION_TITLE = "Hlídač ochranných známek"
AUTOMATION_CRON = "0 7 * * *"
AUTOMATION_COMMAND = "znamky-cli sledovani check"

VALID_KINDS = ("word", "figurative", "combined")
VALID_TERRITORIES = ("CZ", "EU")
DEFAULT_THRESHOLD = scoring.MEDIUM_THRESHOLD
IMAGE_FETCH_CAP = 25  # max candidate logos downloaded+hashed per check


# ── time ───────────────────────────────────────────────────────────────────


def now_utc() -> str:
    return (
        datetime.datetime.now(datetime.timezone.utc)
        .isoformat(timespec="seconds")
        .replace("+00:00", "Z")
    )


# ── input validation ──────────────────────────────────────────────────────────


def normalize_text(text: str) -> str:
    return re.sub(r"\s+", " ", (text or "").strip())


def parse_nice_classes(value) -> list:
    """Accept a list or a 'comma/space separated' string of Nice classes (1–45)."""
    if isinstance(value, str):
        parts = re.split(r"[,;\s]+", value.strip())
    else:
        parts = list(value or [])
    out = []
    for p in parts:
        s = str(p).strip()
        if not s:
            continue
        if not s.isdigit() or not (1 <= int(s) <= 45):
            raise InvalidMarkError(f"Neplatná třída výrobků/služeb (Nice): {p!r}. Očekává se 1–45.")
        out.append(int(s))
    return sorted(set(out))


def parse_territories(value) -> list:
    if isinstance(value, str):
        parts = re.split(r"[,;\s]+", value.strip())
    else:
        parts = list(value or [])
    out = []
    for p in parts:
        s = str(p).strip().upper()
        if not s:
            continue
        if s not in VALID_TERRITORIES:
            raise InvalidMarkError(f"Neplatné území: {p!r}. Povoleno: CZ, EU.")
        out.append(s)
    return out or list(VALID_TERRITORIES)


# ── state file I/O ───────────────────────────────────────────────────────────


def _mark_dir(mark_uuid: str) -> str:
    return os.path.join(APP_DIR, mark_uuid)


def _state_path(mark_uuid: str) -> str:
    return os.path.join(_mark_dir(mark_uuid), "state.json")


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
    """All watched-mark states; marks needing attention (unconfirmed/high) first."""
    if not os.path.isdir(APP_DIR):
        return []
    results = []
    for name in os.listdir(APP_DIR):
        state = _load_state_dir(name)
        if state is not None:
            results.append(state)

    def _sort_key(s):
        collisions = s.get("collisions", [])
        unconfirmed = [c for c in collisions if not c.get("confirmed_on")]
        has_high = any(c.get("tier") == "high" for c in unconfirmed)
        latest = max((c.get("detected_on", "") for c in collisions), default="")
        if unconfirmed:
            tier = 3 if has_high else 2
        elif collisions:
            tier = 1
        else:
            tier = 0
        return (tier, max(latest, s.get("added_on", "")))

    results.sort(key=_sort_key, reverse=True)
    return results


def find_by_uuid(target_uuid: str) -> Optional[dict]:
    if not target_uuid or not os.path.isdir(APP_DIR):
        return None
    state = _load_state_dir(target_uuid)
    if state and state.get("uuid") == target_uuid:
        return state
    for name in os.listdir(APP_DIR):
        s = _load_state_dir(name)
        if s and s.get("uuid") == target_uuid:
            return s
    return None


# ── logo storage + hashing ──────────────────────────────────────────────────


def _ext_for_mime(mime: str) -> str:
    return {
        "image/png": ".png",
        "image/jpeg": ".jpg",
        "image/jpg": ".jpg",
        "image/gif": ".gif",
        "image/webp": ".webp",
        "image/svg+xml": ".svg",
    }.get((mime or "").lower(), ".png")


def store_logo(mark_uuid: str, raw: bytes, mime: str = "image/png") -> str:
    """Persist a logo for a watched mark; return its absolute path."""
    path = os.path.join(_mark_dir(mark_uuid), "logo" + _ext_for_mime(mime))
    atomic_write_bytes(path, raw)
    return path


def _store_candidate_logo(mark_uuid: str, candidate: dict, raw: bytes) -> str:
    safe_id = re.sub(r"[^A-Za-z0-9._-]+", "_", candidate.get("source_id", "") or "x")
    path = os.path.join(_mark_dir(mark_uuid), "candidates", f"{safe_id}.img")
    atomic_write_bytes(path, raw)
    return path


def _hash_logo(path: str) -> dict:
    """Compute the perceptual hash + colour signature of a stored logo (best-effort)."""
    return {
        "logo_phash": image_similarity.perceptual_hash(path),
        "logo_color": image_similarity.color_signature(path),
    }


# ── scoring pipeline ──────────────────────────────────────────────────────────


def _score_candidates(state: dict, candidates: list) -> list:
    """Score every candidate; enrich the top figurative ones with image hashes.

    Returns a list of {"candidate": c, "score": {...}, "tier": ...} sorted by
    overall score descending.
    """
    kind = state.get("kind", "word")
    prelim = []
    for c in candidates:
        result = scoring.score_candidate(state, c)
        prelim.append((c, result))
    prelim.sort(key=lambda pr: pr[1]["scores"]["overall"], reverse=True)

    need_images = (
        kind in ("figurative", "combined")
        and image_similarity.HAS_IMAGE_LIBS
        and state.get("logo_phash")
    )
    if need_images:
        fetched = 0
        for idx, (c, _) in enumerate(prelim):
            if fetched >= IMAGE_FETCH_CAP:
                break
            if not c.get("image_url"):
                continue
            raw = sources.tmview.fetch_image(c)
            if not raw:
                continue
            path = _store_candidate_logo(state["uuid"], c, raw)
            hashed = _hash_logo(path)
            c["logo_phash"] = hashed["logo_phash"]
            c["logo_color"] = hashed["logo_color"]
            c["image_path"] = path
            prelim[idx] = (c, scoring.score_candidate(state, c))
            fetched += 1
        prelim.sort(key=lambda pr: pr[1]["scores"]["overall"], reverse=True)

    return [
        {"candidate": c, "scores": r["scores"], "tier": r["tier"]}
        for c, r in prelim
    ]


def _collision_from(candidate: dict, scores: dict, tier: str) -> dict:
    return {
        "detected_on": now_utc(),
        "source": candidate.get("source", ""),
        "source_id": candidate.get("source_id", ""),
        "mark_text": candidate.get("mark_text", ""),
        "mark_kind": candidate.get("mark_kind", ""),
        "applicant": candidate.get("applicant", ""),
        "status": candidate.get("status", ""),
        "filing_date": candidate.get("filing_date", ""),
        "nice_classes": candidate.get("nice_classes", []),
        "vienna_codes": candidate.get("vienna_codes", []),
        "territory": candidate.get("territory", ""),
        "office": candidate.get("office", ""),
        "url_detail": candidate.get("url_detail", ""),
        "image_url": candidate.get("image_url", ""),
        "image_path": candidate.get("image_path"),
        "scores": scores,
        "tier": tier,
        "ai_assessment": None,
        "confirmed_on": None,
    }


def _collision_id(collision: dict) -> tuple:
    return (collision.get("source", ""), collision.get("source_id", ""))


def _run_search(state: dict) -> dict:
    return sources.search_all(
        state.get("text", ""),
        nice_classes=state.get("nice_classes"),
        territories=state.get("territories"),
        limit=80,
    )


# ── high-level operations ────────────────────────────────────────────────────


def _new_mark_skeleton(kind: str, label: str) -> dict:
    return {
        "uuid": str(uuid_mod.uuid4()),
        "kind": kind,
        "label": label or "",
        "text": "",
        "logo_path": None,
        "logo_phash": None,
        "logo_color": None,
        "nice_classes": [],
        "vienna_codes": [],
        "territories": list(VALID_TERRITORIES),
        "owner_name": "",
        "threshold": DEFAULT_THRESHOLD,
        "added_on": now_utc(),
        "last_check_at": now_utc(),
        "known_collision_ids": [],
        "collisions": [],
    }


def _baseline_collisions(state: dict) -> dict:
    """Initial search at add-time: record current conflicts (unconfirmed) as baseline."""
    search = _run_search(state)
    scored = _score_candidates(state, search["candidates"])
    threshold = state.get("threshold", DEFAULT_THRESHOLD)
    known = set()
    collisions = []
    for item in scored:
        if item["scores"]["overall"] < threshold:
            continue
        collision = _collision_from(item["candidate"], item["scores"], item["tier"])
        collisions.append(collision)
        known.add(_collision_id(collision))
    state["collisions"] = collisions
    state["known_collision_ids"] = ["|".join(k) for k in known]
    state["last_check_at"] = now_utc()
    return {"ok": not search["errors"], "errors": search["errors"], "found": len(collisions)}


def add_text(text, nice_classes=None, territories=None, owner_name="", label="", threshold=None) -> dict:
    """Watch a word mark by its verbal element."""
    verbal = normalize_text(text)
    if len(verbal) < 1:
        raise InvalidMarkError("Text slovní známky je povinný.")
    state = _new_mark_skeleton("word", label)
    state["text"] = verbal
    state["nice_classes"] = parse_nice_classes(nice_classes)
    state["territories"] = parse_territories(territories)
    state["owner_name"] = normalize_text(owner_name)
    if threshold is not None:
        state["threshold"] = float(threshold)
    _baseline_collisions(state)
    _save_state(state)
    ensure_automation()
    return state


def add_logo(logo_bytes, mime="image/png", text="", nice_classes=None, territories=None,
             vienna_codes=None, owner_name="", label="", threshold=None) -> dict:
    """Watch a figurative (or combined, if text given) mark by its logo image."""
    if not logo_bytes:
        raise InvalidMarkError("Obrázek loga je povinný u obrazové známky.")
    verbal = normalize_text(text)
    kind = "combined" if verbal else "figurative"
    state = _new_mark_skeleton(kind, label)
    state["text"] = verbal
    state["nice_classes"] = parse_nice_classes(nice_classes)
    state["territories"] = parse_territories(territories)
    state["vienna_codes"] = [str(c).strip() for c in (vienna_codes or []) if str(c).strip()]
    state["owner_name"] = normalize_text(owner_name)
    if threshold is not None:
        state["threshold"] = float(threshold)

    logo_path = store_logo(state["uuid"], logo_bytes, mime)
    state["logo_path"] = logo_path
    hashed = _hash_logo(logo_path)
    state["logo_phash"] = hashed["logo_phash"]
    state["logo_color"] = hashed["logo_color"]

    _baseline_collisions(state)
    _save_state(state)
    ensure_automation()
    return state


def check_one(mark_uuid: str) -> dict:
    """Re-search the registers for one watched mark; record new collisions."""
    state = find_by_uuid(mark_uuid)
    if state is None:
        raise MarkNotTrackedError("Známka není sledována.")
    try:
        search = _run_search(state)
    except ZnamkyError as exc:
        return {"uuid": mark_uuid, "display_name": display_name(state),
                "ok": False, "new_collisions": [], "error": str(exc)}

    scored = _score_candidates(state, search["candidates"])
    threshold = state.get("threshold", DEFAULT_THRESHOLD)
    known = set(state.get("known_collision_ids", []))
    new_collisions = []
    for item in scored:
        if item["scores"]["overall"] < threshold:
            continue
        collision = _collision_from(item["candidate"], item["scores"], item["tier"])
        key = "|".join(_collision_id(collision))
        if key in known:
            continue
        known.add(key)
        new_collisions.append(collision)

    state["collisions"] = new_collisions + state.get("collisions", [])
    state["known_collision_ids"] = sorted(known)
    state["last_check_at"] = now_utc()
    _save_state(state)

    error = "; ".join(e["error"] for e in search["errors"]) or None
    return {"uuid": mark_uuid, "display_name": display_name(state),
            "ok": True, "new_collisions": new_collisions, "error": error}


def check_all() -> list:
    results = []
    for state in list_all():
        try:
            results.append(check_one(state["uuid"]))
        except ZnamkyError as exc:
            results.append({"uuid": state.get("uuid", "?"), "display_name": display_name(state),
                            "ok": False, "new_collisions": [], "error": str(exc)})
    return results


def confirm(mark_uuid: str, change_index: Optional[int] = None) -> int:
    """Mark collisions as reviewed. Returns the number marked."""
    state = find_by_uuid(mark_uuid)
    if state is None:
        raise MarkNotTrackedError("Známka není sledována.")
    collisions = state.get("collisions", [])
    timestamp = now_utc()
    marked = 0
    if change_index is not None:
        if not (0 <= change_index < len(collisions)):
            raise IndexError(f"Index {change_index} je mimo rozsah (0..{len(collisions) - 1}).")
        if not collisions[change_index].get("confirmed_on"):
            collisions[change_index]["confirmed_on"] = timestamp
            marked = 1
    else:
        for c in collisions:
            if not c.get("confirmed_on"):
                c["confirmed_on"] = timestamp
                marked += 1
    if marked:
        _save_state(state)
    return marked


def set_assessment(mark_uuid: str, change_index: int, assessment: dict) -> dict:
    """Attach an AI likelihood-of-confusion assessment to one collision."""
    state = find_by_uuid(mark_uuid)
    if state is None:
        raise MarkNotTrackedError("Známka není sledována.")
    collisions = state.get("collisions", [])
    if not (0 <= change_index < len(collisions)):
        raise IndexError(f"Index {change_index} je mimo rozsah (0..{len(collisions) - 1}).")
    risk = (assessment.get("risk") or "").lower()
    if risk not in ("high", "medium", "low", "none"):
        raise InvalidMarkError("AI posouzení: 'risk' musí být high/medium/low/none.")
    collisions[change_index]["ai_assessment"] = {
        "risk": risk,
        "visual": assessment.get("visual", ""),
        "aural": assessment.get("aural", ""),
        "conceptual": assessment.get("conceptual", ""),
        "goods_similarity": assessment.get("goods_similarity", ""),
        "summary": assessment.get("summary", ""),
        "assessed_on": now_utc(),
    }
    _save_state(state)
    return collisions[change_index]


def set_label(mark_uuid: str, label: str) -> dict:
    state = find_by_uuid(mark_uuid)
    if state is None:
        raise MarkNotTrackedError("Známka není sledována.")
    state["label"] = label or ""
    _save_state(state)
    return state


def update_mark(mark_uuid: str, text=None, nice_classes=None, territories=None,
                owner_name=None, label=None, threshold=None) -> dict:
    """Update editable fields of a watched mark. Only non-None fields change.

    Does not change the mark's kind or stored logo. A new threshold/classes/
    territories take effect on the next check (which is when sources are queried).
    """
    state = find_by_uuid(mark_uuid)
    if state is None:
        raise MarkNotTrackedError("Známka není sledována.")
    if text is not None:
        verbal = normalize_text(text)
        if state.get("kind") == "word" and not verbal:
            raise InvalidMarkError("Text slovní známky je povinný.")
        state["text"] = verbal
    if nice_classes is not None:
        state["nice_classes"] = parse_nice_classes(nice_classes)
    if territories is not None:
        state["territories"] = parse_territories(territories)
    if owner_name is not None:
        state["owner_name"] = normalize_text(owner_name)
    if label is not None:
        state["label"] = normalize_text(label)
    if threshold is not None:
        value = float(threshold)
        if not (0.0 < value < 1.0):
            raise InvalidMarkError("Práh shody musí být mezi 0 a 1 (např. 0.6).")
        state["threshold"] = round(value, 2)
    _save_state(state)
    return state


def remove(mark_uuid: str) -> None:
    state = find_by_uuid(mark_uuid)
    if state is None:
        raise MarkNotTrackedError("Známka není sledována.")
    target = _mark_dir(state["uuid"])
    if os.path.isdir(target):
        trash = f"{target}.deleted-{uuid_mod.uuid4().hex}"
        os.rename(target, trash)
        shutil.rmtree(trash, ignore_errors=True)


def show(mark_uuid: str) -> dict:
    state = find_by_uuid(mark_uuid)
    if state is None:
        raise MarkNotTrackedError("Známka není sledována.")
    return state


# ── one-shot lookups (no persisted state) ─────────────────────────────────────


def lookup_text(text, nice_classes=None, territories=None, limit=80) -> dict:
    """Ad-hoc word-mark similarity search; returns scored candidates, saves nothing."""
    state = {
        "kind": "word",
        "uuid": "_lustrace",
        "text": normalize_text(text),
        "nice_classes": parse_nice_classes(nice_classes),
        "territories": parse_territories(territories),
        "threshold": 0.0,
    }
    if not state["text"]:
        raise InvalidMarkError("Text pro lustraci je povinný.")
    search = sources.search_all(state["text"], state["nice_classes"], state["territories"], limit)
    scored = _score_candidates(state, search["candidates"])
    return {"query": state["text"], "candidates": scored, "errors": search["errors"]}


def lookup_logo(logo_bytes, mime="image/png", text="", nice_classes=None,
                territories=None, vienna_codes=None, limit=80) -> dict:
    """Ad-hoc figurative similarity search; returns scored candidates, saves nothing."""
    if not logo_bytes:
        raise InvalidMarkError("Obrázek loga je povinný pro obrazovou lustraci.")
    verbal = normalize_text(text)
    state = _new_mark_skeleton("combined" if verbal else "figurative", "")
    state["uuid"] = "_lustrace-" + uuid_mod.uuid4().hex
    state["text"] = verbal
    state["nice_classes"] = parse_nice_classes(nice_classes)
    state["territories"] = parse_territories(territories)
    state["vienna_codes"] = [str(c).strip() for c in (vienna_codes or []) if str(c).strip()]
    try:
        path = store_logo(state["uuid"], logo_bytes, mime)
        hashed = _hash_logo(path)
        state["logo_path"] = path
        state["logo_phash"] = hashed["logo_phash"]
        state["logo_color"] = hashed["logo_color"]
        search = _run_search(state)
        scored = _score_candidates(state, search["candidates"])
        return {"candidates": scored, "errors": search["errors"]}
    finally:
        shutil.rmtree(_mark_dir(state["uuid"]), ignore_errors=True)


# ── automation ───────────────────────────────────────────────────────────────


def ensure_automation() -> None:
    """Ensure the central daily cron automation exists. Idempotent, non-fatal."""
    try:
        result = subprocess.run(
            [CDXCTL_BIN, "automation", "list"],
            capture_output=True, text=True, timeout=15,
        )
        if result.returncode != 0:
            return
        automations = json.loads(result.stdout)
    except Exception:  # noqa: BLE001
        return
    if not isinstance(automations, list):
        return
    for a in automations:
        if isinstance(a, dict) and a.get("type") == "COMMAND" and a.get("command") == AUTOMATION_COMMAND:
            return
    try:
        subprocess.run(
            [CDXCTL_BIN, "automation", "create-command",
             "--title", AUTOMATION_TITLE, "--cron", AUTOMATION_CRON,
             "--command", AUTOMATION_COMMAND,
             "--description", "Pravidelná kontrola podobných ochranných známek (ÚPV, EUIPO přes TMview)."],
            capture_output=True, text=True, timeout=15,
        )
    except Exception:  # noqa: BLE001
        return


# ── presentation helpers ─────────────────────────────────────────────────────


_MIME_BY_EXT = {
    ".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg",
    ".gif": "image/gif", ".webp": "image/webp", ".svg": "image/svg+xml",
}


def logo_data_url(path) -> Optional[str]:
    """Read a stored image and return a base64 data: URL, or None.

    Logos are delivered inline (data URL in the JSON payload) rather than as a
    binary CGI response, because the plugin CGI runtime decodes process stdout as
    UTF-8 text — which corrupts raw image bytes.
    """
    if not path or not os.path.isfile(path):
        return None
    try:
        with open(path, "rb") as f:
            raw = f.read()
    except OSError:
        return None
    mime = _MIME_BY_EXT.get(os.path.splitext(path)[1].lower(), "image/png")
    return f"data:{mime};base64," + base64.b64encode(raw).decode("ascii")


def display_name(state: dict) -> str:
    if state.get("label"):
        return state["label"]
    if state.get("text"):
        return state["text"]
    return "(obrazová známka)"


def _worst_tier(collisions: list) -> Optional[str]:
    order = {"high": 3, "medium": 2, "low": 1}
    best = 0
    label = None
    for c in collisions:
        rank = order.get(c.get("tier", ""), 0)
        if rank > best:
            best = rank
            label = c.get("tier")
    return label


def state_to_overview_entry(state: dict) -> dict:
    collisions = state.get("collisions", [])
    unconfirmed = [c for c in collisions if not c.get("confirmed_on")]
    return {
        "uuid": state.get("uuid", ""),
        "kind": state.get("kind", ""),
        "label": state.get("label", ""),
        "display_name": display_name(state),
        "text": state.get("text", ""),
        "has_logo": bool(state.get("logo_path")),
        "logo_data_url": logo_data_url(state.get("logo_path")),
        "nice_classes": state.get("nice_classes", []),
        "territories": state.get("territories", []),
        "owner_name": state.get("owner_name", ""),
        "added_on": state.get("added_on", ""),
        "last_check_at": state.get("last_check_at", ""),
        "collisions_count": len(collisions),
        "unconfirmed_count": len(unconfirmed),
        "worst_tier": _worst_tier(unconfirmed) or _worst_tier(collisions),
    }


def state_to_detail_entry(state: dict) -> dict:
    entry = state_to_overview_entry(state)
    entry["threshold"] = state.get("threshold", DEFAULT_THRESHOLD)
    entry["vienna_codes"] = state.get("vienna_codes", [])
    entry["collisions"] = [
        {**c, "image_data_url": logo_data_url(c.get("image_path"))}
        for c in state.get("collisions", [])
    ]
    return entry
