"""State I/O: per-document state.json, groups.json, related baseline files.

All writes go through atomic_write_json (tmp file + rename) to keep the visible
state valid even if a write is interrupted.
"""

import datetime
import json
import os
import re
import tempfile
import unicodedata

_USER_HOME = os.environ.get("CDX_USER_HOME") or os.path.expanduser("~")
APP_DIR = os.path.join(_USER_HOME, ".cdx", "apps", "sledovane-dokumenty")
GROUPS_PATH = os.path.join(APP_DIR, "groups.json")


def now_utc():
    """Current UTC timestamp in ISO 8601 'Z' form."""
    return datetime.datetime.now(datetime.timezone.utc).isoformat(
        timespec="seconds"
    ).replace("+00:00", "Z")


# ── atomic writes ────────────────────────────────────────────────────────────


def atomic_write_json(path, data):
    """Write JSON atomically (tmp file + rename)."""
    d = os.path.dirname(path) or "."
    os.makedirs(d, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=".tmp.", suffix=".json", dir=d)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
            f.write("\n")
        os.replace(tmp, path)
    except Exception:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


# ── state (per-document) ─────────────────────────────────────────────────────


def state_dir(codexis_id):
    return os.path.join(APP_DIR, codexis_id)


def state_path(codexis_id):
    return os.path.join(state_dir(codexis_id), "state.json")


def load_state(codexis_id):
    """Return state dict, or None if missing or corrupt."""
    path = state_path(codexis_id)
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError):
        return None


def save_state(codexis_id, state):
    os.makedirs(state_dir(codexis_id), exist_ok=True)
    atomic_write_json(state_path(codexis_id), state)


def all_tracked_ids():
    """Return sorted list of codexis IDs with a readable state.json."""
    if not os.path.isdir(APP_DIR):
        return []
    ids = []
    for name in sorted(os.listdir(APP_DIR)):
        if load_state(name) is not None:
            ids.append(name)
    return ids


# ── groups ───────────────────────────────────────────────────────────────────


def slugify(name):
    """Convert group name to a URL-friendly id."""
    nfkd = unicodedata.normalize("NFKD", name)
    ascii_str = nfkd.encode("ascii", "ignore").decode("ascii").lower()
    slug = re.sub(r"[^a-z0-9]+", "-", ascii_str).strip("-")
    return slug or "skupina"


def load_groups():
    """Return groups list, or [] if missing or corrupt."""
    if not os.path.isfile(GROUPS_PATH):
        return []
    try:
        with open(GROUPS_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, list) else []
    except (json.JSONDecodeError, OSError):
        return []


def save_groups(groups):
    atomic_write_json(GROUPS_PATH, groups)


def find_group(groups, name):
    """Find group by name (case-insensitive). Returns (index, group) or (-1, None)."""
    for i, g in enumerate(groups):
        if g.get("name", "").lower() == name.lower():
            return i, g
    return -1, None


# ── related baselines ────────────────────────────────────────────────────────


def related_baseline_path(codexis_id, relation_type):
    return os.path.join(state_dir(codexis_id), f"related_{relation_type}.json")


def load_related_baseline(codexis_id, relation_type):
    """Return baseline dict, or None if missing."""
    path = related_baseline_path(codexis_id, relation_type)
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError):
        return None


def save_related_baseline(codexis_id, relation_type, data):
    atomic_write_json(related_baseline_path(codexis_id, relation_type), data)


def delete_related_baselines(codexis_id, types=None):
    """Delete baseline files. If types is None, delete all related_*.json in the dir."""
    d = state_dir(codexis_id)
    if not os.path.isdir(d):
        return
    if types is None:
        for fname in os.listdir(d):
            if fname.startswith("related_") and fname.endswith(".json"):
                try:
                    os.remove(os.path.join(d, fname))
                except OSError:
                    pass
    else:
        for rtype in types:
            path = related_baseline_path(codexis_id, rtype)
            if os.path.isfile(path):
                try:
                    os.remove(path)
                except OSError:
                    pass
