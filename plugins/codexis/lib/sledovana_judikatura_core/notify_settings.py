"""Per-app notify settings: shipped defaults merged with a sparse user override.

Two layers, on two different trees:
- default  ->  ~/.local/share/codexis/settings/<APP_ID>/defaults.json
               Install/home tree (plain ``~``, not CODEXIS_PUBLIC_USER_HOME):
               read-only, delivered by hooks/install-binaries.sh next to the
               lib, refreshed on update.
- user     ->  <APP_DIR>/settings.json  (only the keys the user changed)
               User-data tree (state.APP_DIR, i.e. CODEXIS_PUBLIC_USER_HOME
               or ``~`` + .cdx/app-state/<APP_ID>).

Effective config is computed at read time; nothing is merged into the user file.
"""

import copy
import json
import os
import re

from . import state

APP_ID = "sledovana-judikatura"
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


def defaults_path():
    # Install/home tree: defaults.json is delivered by install-binaries.sh
    # using plain ${HOME}, next to the lib itself (which the bin locates via
    # Path.home()). Use os.path.expanduser("~") here rather than
    # state._USER_HOME (which honors CODEXIS_PUBLIC_USER_HOME) so this reads
    # from the same home the installer wrote it to.
    return os.path.join(
        os.path.expanduser("~"),
        ".local",
        "share",
        "codexis",
        "settings",
        APP_ID,
        "defaults.json",
    )


def user_path():
    return os.path.join(state.APP_DIR, "settings.json")


def _read_json(path):
    try:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else None
    except (OSError, ValueError):
        # ValueError covers json.JSONDecodeError and UnicodeDecodeError, so a
        # missing or corrupt file falls back instead of raising.
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
        # No breaking migrations yet; future version bumps add ordered transforms here.
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
    state.atomic_write_json(user_path(), override)
    return get_notify()


def ensure_settings():
    path = user_path()
    existing = _read_json(path)
    if existing is None:
        # First run (or unreadable file): write a minimal, sparse file. Unset
        # keys resolve to defaults.json live on every load(), so a changed
        # default reaches this user instead of being frozen in.
        state.atomic_write_json(path, {"version": SCHEMA_VERSION})
        return
    if migrate(existing):
        state.atomic_write_json(path, existing)
