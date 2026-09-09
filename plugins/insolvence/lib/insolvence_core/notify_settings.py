"""Per-app notify settings: shipped defaults merged with a sparse user override.

- default -> ~/.local/share/<APP_ID>/settings/defaults.json (delivered by install hook, sibling of the plugin's lib/)
- user    -> ${CODEXIS_PUBLIC_USER_HOME|~}/.cdx/app-state/<APP_ID>/settings.json (only changed keys)

Effective config is computed at read time; nothing is merged into the user file.
"""

import copy
import json
import os
import re

from .fs import atomic_write_json

APP_ID = "insolvence"
SCHEMA_VERSION = 1
FALLBACK_DEFAULTS = {
    "version": SCHEMA_VERSION,
    "notify": {"email": False, "inApp": True, "recipients": []},
}

_EMAIL_RE = re.compile(r"^[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}$")


def clean_recipients(recipients):
    """Trim, validate format, dedupe (case-insensitive, order-preserving).

    Raises ValueError on an invalid address. Empty entries are dropped.
    """
    seen = set()
    out = []
    for r in recipients:
        addr = str(r).strip()
        if not addr:
            continue
        if not _EMAIL_RE.match(addr):
            raise ValueError(f"Neplatná e-mailová adresa: {addr}")
        key = addr.lower()
        if key in seen:
            continue
        seen.add(key)
        out.append(addr)
    return out


def _user_home():
    return os.environ.get("CODEXIS_PUBLIC_USER_HOME") or os.path.expanduser("~")


def defaults_path():
    return os.path.join(
        os.path.expanduser("~"), ".local", "share", APP_ID, "settings", "defaults.json"
    )


def user_path():
    return os.path.join(_user_home(), ".cdx", "app-state", APP_ID, "settings.json")


def _read_json(path):
    try:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else None
    except (OSError, ValueError):
        # ValueError covers JSONDecodeError + UnicodeDecodeError: corrupt/missing
        # file falls back instead of raising.
        return None


def load_defaults():
    data = _read_json(defaults_path())
    return data if data is not None else copy.deepcopy(FALLBACK_DEFAULTS)


def deep_merge(base, override):
    out = dict(base)
    for key, value in override.items():
        if isinstance(value, dict) and isinstance(out.get(key), dict):
            out[key] = deep_merge(out[key], value)
        else:
            out[key] = value
    return out


def load():
    override = _read_json(user_path()) or {}
    return deep_merge(load_defaults(), override)


def migrate(user):
    changed = False
    try:
        v = int(user.get("version", 1))
    except (TypeError, ValueError):
        v = 0
    if v < SCHEMA_VERSION:
        user["version"] = SCHEMA_VERSION
        changed = True
    return changed


def get_notify():
    return load()["notify"]


def set_notify(email=None, inApp=None, recipients=None):
    override = _read_json(user_path()) or {}
    notify = override.get("notify")
    notify = dict(notify) if isinstance(notify, dict) else {}
    if email is not None:
        notify["email"] = bool(email)
    if inApp is not None:
        notify["inApp"] = bool(inApp)
    if recipients is not None:
        notify["recipients"] = clean_recipients(recipients)
    override["notify"] = notify
    atomic_write_json(user_path(), override)
    return get_notify()


def ensure_settings():
    path = user_path()
    existing = _read_json(path)
    if existing is None:
        atomic_write_json(path, {"version": SCHEMA_VERSION})
        return
    if migrate(existing):
        atomic_write_json(path, existing)
