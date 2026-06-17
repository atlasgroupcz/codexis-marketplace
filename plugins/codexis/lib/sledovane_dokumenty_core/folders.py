"""Watched folders: scan a document tree, keep the .watched/watched.json contract.

A *watched folder* is a directory of the user's own documents. For each watched
root we keep a hidden ``.watched/`` directory containing:

- ``watched.json``        — the stable, documented integration contract (see
  ``skills/sledovane-dokumenty/references/watched-json-contract.md``). Holds one
  entry per document: relative path, SHA-256, ISO-8601 timestamps and the
  AI-harvested ``legislation`` references.
- ``tracking-state.json`` — deterministic change-tracking bookkeeping written by
  the daily runner (CODEXIS baseline version per referenced act). Never touched
  by AI.

The list of watched roots lives in ``<APP_DIR>/watched-folders.json`` so the CLI,
the component handler and the automation all iterate the same set.

All writes go through ``state.atomic_write_json`` (tmp file + rename).
"""

import json
import os
import subprocess
import uuid as uuid_mod

from . import state
from .exceptions import DocumentError
from .state import now_utc

WATCHED_DIRNAME = ".watched"
WATCHED_JSON = "watched.json"
TRACKING_JSON = "tracking-state.json"
CONTRACT_VERSION = 1

# Documents we can extract text from (the daemon extract agent handles docx/pdf/
# txt directly and OCRs the rest).
SUPPORTED_EXTS = {
    ".pdf", ".docx", ".doc", ".rtf", ".odt", ".txt", ".md",
}

CDXCTL_BIN = "cdxctl"
FOLDER_CHECK_AUTOMATION_TITLE = "Sledované složky – kontrola legislativy"
FOLDER_CHECK_AUTOMATION_CRON = "0 6 * * *"
FOLDER_CHECK_AUTOMATION_COMMAND = "cdx-sledovane-dokumenty folder check"
FOLDER_CHECK_AUTOMATION_DESC = (
    "Denní deterministická kontrola změn legislativy odkazované ve sledovaných "
    "složkách."
)


class WatchedFolderError(DocumentError):
    """A watched-folder operation failed (bad root, escape attempt, …)."""


# ── home / path safety ───────────────────────────────────────────────────────


def user_home():
    """Resolve the user's home (read at call time so tests can override env)."""
    return os.environ.get("CODEXIS_PUBLIC_USER_HOME") or os.path.expanduser("~")


def _is_within(base, path):
    """True if `path` is `base` or a descendant of it (both already abspath)."""
    base = os.path.normpath(base)
    path = os.path.normpath(path)
    if path == base:
        return True
    return path.startswith(base + os.sep)


def normalize_root(root):
    """Absolutize a watched root and assert it stays inside the user's home."""
    if not root or not str(root).strip():
        raise WatchedFolderError("Cesta ke složce nesmí být prázdná.")
    abs_root = os.path.abspath(os.path.expanduser(str(root)))
    home = os.path.abspath(user_home())
    if not _is_within(home, abs_root):
        raise WatchedFolderError(
            f"Složka musí být uvnitř domovského adresáře ({home})."
        )
    return abs_root


# ── path helpers ─────────────────────────────────────────────────────────────


def watched_dir(root):
    return os.path.join(root, WATCHED_DIRNAME)


def watched_json_path(root):
    return os.path.join(watched_dir(root), WATCHED_JSON)


def tracking_path(root):
    return os.path.join(watched_dir(root), TRACKING_JSON)


def to_rel(root, abs_path):
    """Relative POSIX path from the watched root, prefixed with ``./``."""
    rel = os.path.relpath(abs_path, root).replace(os.sep, "/")
    return rel if rel.startswith("./") else "./" + rel


def to_abs(root, rel_path):
    """Resolve a ``./sub/file`` relative path back to an absolute path."""
    rel = rel_path[2:] if rel_path.startswith("./") else rel_path
    return os.path.normpath(os.path.join(root, rel))


# ── checksums / scanning ─────────────────────────────────────────────────────


def sha256_file(path):
    import hashlib

    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def scan_files(root):
    """Walk `root`, returning [{path, sha256}] for supported documents.

    Hidden directories (``.watched``, ``.git`` …) and hidden files are skipped.
    Result is sorted by relative path for stable output.
    """
    found = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = sorted(d for d in dirnames if not d.startswith("."))
        for fname in sorted(filenames):
            if fname.startswith("."):
                continue
            ext = os.path.splitext(fname)[1].lower()
            if ext not in SUPPORTED_EXTS:
                continue
            abs_path = os.path.join(dirpath, fname)
            try:
                checksum = sha256_file(abs_path)
            except OSError:
                continue
            found.append({"path": to_rel(root, abs_path), "sha256": checksum})
    found.sort(key=lambda d: d["path"])
    return found


# ── watched.json (the contract) ──────────────────────────────────────────────


def read_watched(root):
    """Return the watched.json dict, or None if missing/corrupt."""
    path = watched_json_path(root)
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError):
        return None


def write_watched(root, data):
    data["generatedAt"] = now_utc()
    state.atomic_write_json(watched_json_path(root), data)


def empty_watched():
    return {
        "version": CONTRACT_VERSION,
        "watchedRoot": ".",
        "generatedAt": now_utc(),
        "documents": [],
    }


def _documents_by_path(data):
    return {d.get("path"): d for d in data.get("documents", []) if d.get("path")}


def sync_watched(root):
    """Reconcile watched.json with the files currently on disk.

    - new file            → add entry (empty legislation, extractedAt=null)
    - same path, new hash  → update sha256, reset extractedAt (re-harvest), keep
      the stale legislation until the next harvest overwrites it
    - removed file         → drop entry

    Returns (data, summary) where summary = {added, changed, removed, total}.
    """
    scanned = scan_files(root)
    existing = read_watched(root) or empty_watched()
    old_by_path = _documents_by_path(existing)
    now = now_utc()

    documents = []
    added = changed = 0
    seen = set()
    for entry in scanned:
        path = entry["path"]
        seen.add(path)
        checksum = entry["sha256"]
        prev = old_by_path.get(path)
        if prev is None:
            documents.append({
                "path": path,
                "sha256": checksum,
                "discoveredAt": now,
                "extractedAt": None,
                "updatedAt": now,
                "legislation": [],
            })
            added += 1
        elif prev.get("sha256") != checksum:
            doc = dict(prev)
            doc["sha256"] = checksum
            doc["extractedAt"] = None  # checksum changed → re-harvest
            doc["updatedAt"] = now
            documents.append(doc)
            changed += 1
        else:
            documents.append(prev)

    removed = sum(1 for p in old_by_path if p not in seen)

    documents.sort(key=lambda d: d["path"])
    data = {
        "version": CONTRACT_VERSION,
        "watchedRoot": ".",
        "generatedAt": now,
        "documents": documents,
    }
    write_watched(root, data)
    return data, {
        "added": added,
        "changed": changed,
        "removed": removed,
        "total": len(documents),
    }


def docs_needing_harvest(root):
    """Relative paths of documents whose references must be (re)harvested."""
    data = read_watched(root)
    if not data:
        return []
    return [
        d["path"] for d in data.get("documents", [])
        if d.get("extractedAt") is None and d.get("path")
    ]


def set_references(root, rel_path, legislation):
    """Persist harvested legislation for one document; stamp extracted/updated.

    `legislation` is a list of ``{"uri", "text", "codexisId"?}`` dicts.
    Returns the updated document entry. Raises if the path is unknown.
    """
    data = read_watched(root)
    if not data:
        raise WatchedFolderError(f"watched.json chybí pro {root}.")
    now = now_utc()
    for doc in data.get("documents", []):
        if doc.get("path") == rel_path:
            doc["legislation"] = legislation
            doc["extractedAt"] = now
            doc["updatedAt"] = now
            write_watched(root, data)
            return doc
    raise WatchedFolderError(f"Dokument {rel_path} není ve watched.json.")


def mark_all_for_reharvest(root):
    """Force a full re-harvest on the next run (explicit user refresh)."""
    data = read_watched(root)
    if not data:
        return 0
    now = now_utc()
    count = 0
    for doc in data.get("documents", []):
        if doc.get("extractedAt") is not None:
            doc["extractedAt"] = None
            doc["updatedAt"] = now
            count += 1
    write_watched(root, data)
    return count


def unique_legislation(root):
    """Distinct legislation entries across all documents (dedup by codexisId|uri)."""
    data = read_watched(root)
    if not data:
        return []
    seen = {}
    for doc in data.get("documents", []):
        for ref in doc.get("legislation", []):
            key = ref.get("codexisId") or ref.get("uri") or ref.get("text")
            if key and key not in seen:
                seen[key] = ref
    return list(seen.values())


# ── tracking-state.json (deterministic runner bookkeeping) ───────────────────


def read_tracking(root):
    path = tracking_path(root)
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError):
        return None


def write_tracking(root, data):
    state.atomic_write_json(tracking_path(root), data)


def empty_tracking():
    return {
        "version": CONTRACT_VERSION,
        "watchedRoot": ".",
        "lastCheckAt": None,
        "legislation": {},
    }


# ── watched-folder index ─────────────────────────────────────────────────────


def index_path():
    return os.path.join(state.APP_DIR, "watched-folders.json")


def settings_path():
    """Path to the folder-watch notification settings file."""
    return os.path.join(state.APP_DIR, "folder-watch-settings.json")


def load_index():
    path = index_path()
    if not os.path.isfile(path):
        return []
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, list) else []
    except (json.JSONDecodeError, OSError):
        return []


def save_index(entries):
    state.atomic_write_json(index_path(), entries)


def find_in_index(root):
    for entry in load_index():
        if entry.get("root") == root:
            return entry
    return None


def add_watch(root, name=None):
    """Register a watched root, scan it, and bootstrap the daily automation.

    Returns {root, name, uuid, ...scan summary}. Idempotent on the root.
    """
    abs_root = normalize_root(root)
    if not os.path.isdir(abs_root):
        raise WatchedFolderError(f"Složka neexistuje: {abs_root}")

    _, summary = sync_watched(abs_root)

    entries = load_index()
    entry = next((e for e in entries if e.get("root") == abs_root), None)
    if entry is None:
        entry = {
            "uuid": str(uuid_mod.uuid4()),
            "root": abs_root,
            "name": name or os.path.basename(abs_root.rstrip(os.sep)) or abs_root,
            "added_on": now_utc(),
        }
        entries.append(entry)
        save_index(entries)

    ensure_folder_check_automation()

    result = dict(entry)
    result.update(summary)
    return result


def remove_watch(root, purge=False):
    """Stop watching a root. With `purge`, also delete its ``.watched/`` dir."""
    abs_root = os.path.abspath(os.path.expanduser(str(root)))
    entries = load_index()
    remaining = [e for e in entries if e.get("root") != abs_root]
    removed = len(remaining) != len(entries)
    if removed:
        save_index(remaining)
    if purge:
        import shutil

        shutil.rmtree(watched_dir(abs_root), ignore_errors=True)
    return removed


def list_watches():
    """Index entries enriched with document/reference/change counts."""
    result = []
    for entry in load_index():
        root = entry.get("root")
        data = read_watched(root) if root else None
        tracking = read_tracking(root) if root else None
        documents = data.get("documents", []) if data else []
        legislation = unique_legislation(root) if root else []
        pending = sum(1 for d in documents if d.get("extractedAt") is None)
        unconfirmed = 0
        if tracking:
            for leg in tracking.get("legislation", {}).values():
                unconfirmed += sum(
                    1 for c in leg.get("changes", []) if not c.get("confirmed_on")
                )
        out = dict(entry)
        out.update({
            "documents": len(documents),
            "pending_harvest": pending,
            "legislation": len(legislation),
            "unconfirmed_changes": unconfirmed,
            "last_check_at": tracking.get("lastCheckAt") if tracking else None,
        })
        result.append(out)
    result.sort(key=lambda e: (e.get("unconfirmed_changes", 0), e.get("added_on", "")), reverse=True)
    return result


# ── directory browsing (folder picker) ───────────────────────────────────────


def browse(path=None):
    """List immediate sub-directories of `path` for the folder picker.

    Defaults to the user's home. Refuses to escape the home directory.
    Returns {path, parent, entries:[{name, path, isDir}]}.
    """
    home = os.path.abspath(user_home())
    target = os.path.abspath(os.path.expanduser(path)) if path else home
    if not _is_within(home, target):
        target = home
    if not os.path.isdir(target):
        raise WatchedFolderError(f"Není adresář: {target}")

    dirs = []
    try:
        for name in sorted(os.listdir(target)):
            if name.startswith("."):
                continue
            full = os.path.join(target, name)
            if os.path.isdir(full):
                dirs.append({"name": name, "path": full, "isDir": True})
    except OSError as exc:
        raise WatchedFolderError(f"Nelze číst adresář {target}: {exc}")

    parent = os.path.dirname(target) if target != home else None
    return {"path": target, "parent": parent, "home": home, "entries": dirs}


# ── automation ───────────────────────────────────────────────────────────────


def ensure_folder_check_automation():
    """Ensure the daily folder-check automation exists. Idempotent, non-fatal."""
    try:
        result = subprocess.run(
            [CDXCTL_BIN, "automation", "list"],
            capture_output=True, text=True, timeout=15,
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
            and a.get("command") == FOLDER_CHECK_AUTOMATION_COMMAND
        ):
            return

    try:
        subprocess.run(
            [
                CDXCTL_BIN, "automation", "create-command",
                "--title", FOLDER_CHECK_AUTOMATION_TITLE,
                "--cron", FOLDER_CHECK_AUTOMATION_CRON,
                "--command", FOLDER_CHECK_AUTOMATION_COMMAND,
                "--description", FOLDER_CHECK_AUTOMATION_DESC,
            ],
            capture_output=True, text=True, timeout=15,
        )
    except Exception:
        pass
