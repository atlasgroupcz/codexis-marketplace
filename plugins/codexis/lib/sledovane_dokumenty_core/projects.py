"""Watched projects: scan a project's documents, keep the watched.json contract.

A *watched project* is one of the user's daemon projects. Its documents are the
project's files. The daemon shows them at a different place depending on where
the caller runs:

- on the raw VM (the component's CGI handler): ``~/.cdx/project-home/<id>/files``
- in a chat that belongs to the project: ``/home/codexis/project/files``
- anywhere else (another chat, an automation): not reachable at all

The daemon's extract endpoint reads a document inside a chat's environment, so
harvesting always names the in-chat path and borrows a chat of the project as
its session - the current chat when it belongs to the project, otherwise the
project's newest chat.

For each watched project we keep ``<APP_DIR>/projects/<projectId>/``:

- ``watched.json``        - the stable, documented integration contract (see
  ``skills/sledovane-dokumenty/references/watched-json-contract.md``). Holds one
  entry per document: path relative to the project's files, SHA-256, ISO-8601
  timestamps and the AI-harvested ``legislation`` references.
- ``tracking-state.json`` - deterministic change-tracking bookkeeping written by
  the daily runner (CODEXIS baseline version per referenced act). Never touched
  by AI.

The list of watched projects lives in ``<APP_DIR>/watched-projects.json`` so the
CLI, the component handler and the automation all iterate the same set.

All writes go through ``state.atomic_write_json`` (tmp file + rename).
"""

import json
import os
import uuid as uuid_mod

from . import _daemon, nodeids, state
from .exceptions import DocumentError
from .state import now_utc

WATCHED_JSON = "watched.json"
TRACKING_JSON = "tracking-state.json"
PROJECTS_DIRNAME = "projects"
INDEX_FILE = "watched-projects.json"
CONTRACT_VERSION = 2

PROJECT_HOME_DIRNAME = os.path.join(".cdx", "project-home")
FILES_DIRNAME = "files"
MOUNTED_FILES_DIR = "/home/codexis/project/files"

PROJECT_TYPE = "Project"
CHAT_TYPE = "Chat"

# Documents we can extract text from (the daemon extract agent handles docx/pdf/
# txt directly and OCRs the rest).
SUPPORTED_EXTS = {
    ".pdf",
    ".docx",
    ".doc",
    ".rtf",
    ".odt",
    ".txt",
    ".md",
}

_PROJECTS_QUERY = "query { projects { id name } }"
_CHAT_PROJECT_QUERY = (
    "query ($id: ID!) { node(id: $id) { ... on Chat { project { id name } } } }"
)
_PROJECT_CHATS_QUERY = (
    "query ($id: ID!) { node(id: $id) { ... on Project { chats(limit: 1) { id } } } }"
)


class WatchedProjectError(DocumentError):
    """A watched-project operation failed (unknown project, files unreachable, …)."""


# ── home / paths ─────────────────────────────────────────────────────────────


def user_home():
    """Resolve the user's home (read at call time so tests can override env)."""
    return os.environ.get("CODEXIS_PUBLIC_USER_HOME") or os.path.expanduser("~")


def state_root():
    return os.path.join(state.APP_DIR, PROJECTS_DIRNAME)


def state_dir(project_id):
    return os.path.join(state_root(), project_id)


def watched_json_path(project_id):
    return os.path.join(state_dir(project_id), WATCHED_JSON)


def tracking_path(project_id):
    return os.path.join(state_dir(project_id), TRACKING_JSON)


def index_path():
    return os.path.join(state.APP_DIR, INDEX_FILE)


def to_rel(root, abs_path):
    """POSIX path relative to the project's files, prefixed with ``./``."""
    rel = os.path.relpath(abs_path, root).replace(os.sep, "/")
    return rel if rel.startswith("./") else "./" + rel


def to_abs(root, rel_path):
    """Resolve a ``./sub/file`` relative path against a files directory."""
    rel = rel_path[2:] if rel_path.startswith("./") else rel_path
    return os.path.normpath(os.path.join(root, rel))


def harvest_path(rel_path):
    """The document's path inside a chat of the project - what the daemon reads."""
    return to_abs(MOUNTED_FILES_DIR, rel_path)


# ── daemon lookups ───────────────────────────────────────────────────────────


def _project_from_node(node):
    if not isinstance(node, dict):
        return None
    project_id = nodeids.root_id(node.get("id"), PROJECT_TYPE)
    if not project_id:
        return None
    return {
        "projectId": project_id,
        "nodeId": node["id"],
        "name": node.get("name") or project_id,
    }


def list_available():
    """The user's projects from the daemon, or None when it did not answer."""
    data = _daemon.graphql(_PROJECTS_QUERY)
    if data is None:
        return None
    projects = [_project_from_node(n) for n in data.get("projects") or []]
    return [p for p in projects if p]


def current_session_id():
    return (os.environ.get("CODEXIS_PUBLIC_SESSION_ID") or "").strip() or None


def current_chat_project():
    """The project the current chat belongs to, or None (no chat, no project)."""
    session = current_session_id()
    if not session:
        return None
    data = _daemon.graphql(
        _CHAT_PROJECT_QUERY, {"id": nodeids.encode((CHAT_TYPE, session))}
    )
    node = (data or {}).get("node") or {}
    return _project_from_node(node.get("project"))


def _is_uuid(value):
    try:
        uuid_mod.UUID(str(value))
    except (ValueError, AttributeError, TypeError):
        return False
    return True


def resolve(ref):
    """Turn a user-supplied project reference into ``{projectId, name}``.

    Accepts a project UUID, a daemon ``Project`` node id or a project name
    (case-insensitive). An empty reference means the current chat's project.
    """
    ref = (ref or "").strip() if isinstance(ref, str) else ref
    if not ref:
        current = current_chat_project()
        if current:
            return current
        raise WatchedProjectError(
            "Zadejte projekt (název nebo id) - tento chat nepatří do žádného projektu."
        )

    node_project_id = nodeids.root_id(ref, PROJECT_TYPE)
    project_id = node_project_id or (ref if _is_uuid(ref) else None)

    available = list_available()
    if project_id:
        if available is not None:
            for p in available:
                if p["projectId"] == project_id:
                    return p
            raise WatchedProjectError(f"Projekt {project_id} neexistuje.")
        indexed = find_in_index(project_id)
        if indexed:
            return {"projectId": project_id, "name": indexed.get("name") or project_id}
        raise WatchedProjectError("Daemon neodpovídá, projekt nelze ověřit.")

    if available is None:
        raise WatchedProjectError("Daemon neodpovídá, projekt nelze podle názvu najít.")
    matches = [p for p in available if p["name"].strip().lower() == ref.lower()]
    if len(matches) == 1:
        return matches[0]
    if not matches:
        raise WatchedProjectError(f"Projekt „{ref}“ neexistuje.")
    raise WatchedProjectError(
        f"Název „{ref}“ má více projektů, zadejte id: "
        + ", ".join(p["projectId"] for p in matches)
    )


def files_dir(project_id):
    """Where this process can read the project's files, or None if nowhere."""
    raw = os.path.join(user_home(), PROJECT_HOME_DIRNAME, project_id, FILES_DIRNAME)
    if os.path.isdir(raw):
        return raw
    current = current_chat_project()
    if (
        current
        and current["projectId"] == project_id
        and os.path.isdir(MOUNTED_FILES_DIR)
    ):
        return MOUNTED_FILES_DIR
    return None


def require_files_dir(project_id):
    root = files_dir(project_id)
    if root is None:
        raise WatchedProjectError(
            "Soubory projektu nejsou odtud dostupné - spusťte příkaz z chatu v tomto "
            "projektu, nebo extrakci spusťte v aplikaci Sledované dokumenty."
        )
    return root


def harvest_session(project_id):
    """A chat id whose environment mounts the project's files, or None."""
    session = current_session_id()
    if session:
        current = current_chat_project()
        if current and current["projectId"] == project_id:
            return session
    data = _daemon.graphql(
        _PROJECT_CHATS_QUERY, {"id": nodeids.encode((PROJECT_TYPE, project_id))}
    )
    chats = ((data or {}).get("node") or {}).get("chats") or []
    for chat in chats:
        chat_id = nodeids.root_id((chat or {}).get("id"), CHAT_TYPE)
        if chat_id:
            return chat_id
    return None


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

    Hidden directories and hidden files are skipped. Result is sorted by
    relative path for stable output.
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


def read_watched(project_id):
    """Return the watched.json dict, or None if missing/corrupt."""
    return _read_json(watched_json_path(project_id))


def write_watched(project_id, data):
    data["generatedAt"] = now_utc()
    state.atomic_write_json(watched_json_path(project_id), data)


def empty_watched(project_id):
    return {
        "version": CONTRACT_VERSION,
        "projectId": project_id,
        "generatedAt": now_utc(),
        "documents": [],
    }


def _documents_by_path(data):
    return {d.get("path"): d for d in data.get("documents", []) if d.get("path")}


def sync_watched(project_id):
    """Reconcile watched.json with the project's files as they are now.

    - new file            → add entry (empty legislation, extractedAt=null)
    - same path, new hash  → update sha256, reset extractedAt (re-harvest), keep
      the stale legislation until the next harvest overwrites it
    - removed file         → drop entry

    Returns (data, summary) where summary = {added, changed, removed, total}.
    Raises when the project's files are not reachable from this process.
    """
    scanned = scan_files(require_files_dir(project_id))
    existing = read_watched(project_id) or empty_watched(project_id)
    old_by_path = _documents_by_path(existing)
    now = now_utc()

    documents: list[dict] = []
    added = changed = 0
    seen = set()
    for entry in scanned:
        path = entry["path"]
        seen.add(path)
        checksum = entry["sha256"]
        prev = old_by_path.get(path)
        if prev is None:
            documents.append(
                {
                    "path": path,
                    "sha256": checksum,
                    "discoveredAt": now,
                    "extractedAt": None,
                    "updatedAt": now,
                    "legislation": [],
                }
            )
            added += 1
        elif prev.get("sha256") != checksum:
            doc = dict(prev)
            doc["sha256"] = checksum
            doc["extractedAt"] = None
            doc["updatedAt"] = now
            documents.append(doc)
            changed += 1
        else:
            documents.append(prev)

    removed = sum(1 for p in old_by_path if p not in seen)

    documents.sort(key=lambda d: d["path"])
    data = {
        "version": CONTRACT_VERSION,
        "projectId": project_id,
        "generatedAt": now,
        "documents": documents,
    }
    write_watched(project_id, data)
    return data, {
        "added": added,
        "changed": changed,
        "removed": removed,
        "total": len(documents),
    }


def docs_needing_harvest(project_id):
    """Relative paths of documents whose references must be (re)harvested."""
    data = read_watched(project_id)
    if not data:
        return []
    return [
        d["path"]
        for d in data.get("documents", [])
        if d.get("extractedAt") is None and d.get("path")
    ]


def set_references(project_id, rel_path, legislation):
    """Persist harvested legislation for one document; stamp extracted/updated.

    `legislation` is a list of ``{"uri", "text", "codexisId"?}`` dicts.
    Returns the updated document entry. Raises if the path is unknown.
    """
    data = read_watched(project_id)
    if not data:
        raise WatchedProjectError(f"watched.json chybí pro projekt {project_id}.")
    now = now_utc()
    for doc in data.get("documents", []):
        if doc.get("path") == rel_path:
            doc["legislation"] = legislation
            doc["extractedAt"] = now
            doc["updatedAt"] = now
            write_watched(project_id, data)
            return doc
    raise WatchedProjectError(f"Dokument {rel_path} není ve watched.json.")


def mark_all_for_reharvest(project_id):
    """Force a full re-harvest on the next run (explicit user refresh)."""
    data = read_watched(project_id)
    if not data:
        return 0
    now = now_utc()
    count = 0
    for doc in data.get("documents", []):
        if doc.get("extractedAt") is not None:
            doc["extractedAt"] = None
            doc["updatedAt"] = now
            count += 1
    write_watched(project_id, data)
    return count


def unique_legislation(project_id):
    """Distinct legislation entries across all documents (dedup by codexisId|uri)."""
    data = read_watched(project_id)
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


def read_tracking(project_id):
    return _read_json(tracking_path(project_id))


def write_tracking(project_id, data):
    state.atomic_write_json(tracking_path(project_id), data)


def empty_tracking(project_id):
    return {
        "version": CONTRACT_VERSION,
        "projectId": project_id,
        "lastCheckAt": None,
        "legislation": {},
    }


# ── watched-project index ────────────────────────────────────────────────────


def _read_json(path):
    if not os.path.isfile(path):
        return None
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError):
        return None


def load_index():
    data = _read_json(index_path())
    return data if isinstance(data, list) else []


def save_index(entries):
    state.atomic_write_json(index_path(), entries)


def find_in_index(project_id):
    for entry in load_index():
        if entry.get("projectId") == project_id:
            return entry
    return None


def add_watch(ref):
    """Register a project, scan its files when reachable, return the entry.

    Returns {uuid, projectId, name, added_on, files_available, ...scan summary}.
    Idempotent on the project.
    """
    project = resolve(ref)
    project_id = project["projectId"]

    entries = load_index()
    entry = next((e for e in entries if e.get("projectId") == project_id), None)
    if entry is None:
        entry = {
            "uuid": str(uuid_mod.uuid4()),
            "projectId": project_id,
            "name": project["name"],
            "added_on": now_utc(),
        }
        entries.append(entry)
    else:
        entry["name"] = project["name"]
    save_index(entries)

    result = dict(entry)
    if files_dir(project_id) is not None:
        _, summary = sync_watched(project_id)
        result.update(summary)
        result["files_available"] = True
    else:
        if read_watched(project_id) is None:
            write_watched(project_id, empty_watched(project_id))
        result.update({"added": 0, "changed": 0, "removed": 0, "total": 0})
        result["files_available"] = False
    return result


def remove_watch(project_id, purge=False):
    """Stop watching a project. With `purge`, also delete its state directory."""
    entries = load_index()
    remaining = [e for e in entries if e.get("projectId") != project_id]
    removed = len(remaining) != len(entries)
    if removed:
        save_index(remaining)
    if purge:
        import shutil

        shutil.rmtree(state_dir(project_id), ignore_errors=True)
    return removed


def list_watches():
    """Index entries enriched with document/reference/change counts."""
    result = []
    for entry in load_index():
        project_id = entry.get("projectId")
        if not project_id:
            continue
        data = read_watched(project_id)
        tracking = read_tracking(project_id)
        documents = data.get("documents", []) if data else []
        legislation = unique_legislation(project_id)
        pending = sum(1 for d in documents if d.get("extractedAt") is None)
        unconfirmed = 0
        if tracking:
            for leg in tracking.get("legislation", {}).values():
                unconfirmed += sum(
                    1 for c in leg.get("changes", []) if not c.get("confirmed_on")
                )
        out = dict(entry)
        out.update(
            {
                "documents": len(documents),
                "pending_harvest": pending,
                "legislation": len(legislation),
                "unconfirmed_changes": unconfirmed,
                "last_check_at": tracking.get("lastCheckAt") if tracking else None,
            }
        )
        result.append(out)
    result.sort(
        key=lambda e: (e.get("unconfirmed_changes", 0), e.get("added_on", "")),
        reverse=True,
    )
    return result


def available_to_watch():
    """Projects the user could still add: daemon projects minus the watched ones."""
    available = list_available()
    if available is None:
        raise WatchedProjectError(
            "Daemon neodpovídá, seznam projektů není k dispozici."
        )
    watched = {e.get("projectId") for e in load_index()}
    return [p for p in available if p["projectId"] not in watched]
