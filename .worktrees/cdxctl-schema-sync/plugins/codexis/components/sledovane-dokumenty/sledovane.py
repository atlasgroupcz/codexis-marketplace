#!/usr/bin/env python3
"""JSON CGI handler for the sledovane-dokumenty plugin component.

GET returns JSON data; POST dispatches JSON actions (confirm / remove /
notes / groups). The HTML shell (index.html) is served by the daemon's
SpaCgi runtime directly from `entrypoint` in component.json, so this
module only runs for JSON requests.

State lives in $CODEXIS_PUBLIC_USER_HOME/.cdx/apps/sledovane-dokumenty/ (managed by
sledovane_dokumenty_core).
"""

import json
import os
import sys
import urllib.parse
from pathlib import Path

HERE = Path(__file__).resolve().parent
PLUGIN_DIR = HERE.parent.parent
LIB_DIR = PLUGIN_DIR / "lib"

sys.path.insert(0, str(LIB_DIR))

try:
    from sledovane_dokumenty_core import (
        folder_check,
        folders,
        notify,
        state,
        tracking,
    )
    from sledovane_dokumenty_core.exceptions import (
        DocumentAlreadyTrackedError,
        DocumentError,
        DocumentNotFoundError,
        DocumentNotTrackedError,
        GroupNotFoundError,
    )
    from sledovane_dokumenty_core.folders import WatchedFolderError
except ImportError as e:
    sys.stdout.write("Status: 500 Internal Server Error\r\n")
    sys.stdout.write("Content-Type: application/json; charset=utf-8\r\n\r\n")
    sys.stdout.write(json.dumps({"error": f"sledovane_dokumenty_core import failed: {e}"}))
    sys.exit(0)


# ── output helpers ──────────────────────────────────────────────────────────


def emit_json(payload, status=None):
    if status:
        sys.stdout.write(f"Status: {status}\r\n")
    sys.stdout.write("Content-Type: application/json; charset=utf-8\r\n")
    sys.stdout.write("Cache-Control: no-store\r\n\r\n")
    sys.stdout.write(json.dumps(payload, indent=2, ensure_ascii=False))


def now_utc():
    import datetime
    return datetime.datetime.now(datetime.timezone.utc).isoformat(
        timespec="seconds"
    ).replace("+00:00", "Z")


def error_status(exc):
    if isinstance(exc, DocumentAlreadyTrackedError):
        return "409 Conflict"
    if isinstance(exc, WatchedFolderError):
        return "400 Bad Request"
    if isinstance(exc, (DocumentNotTrackedError, DocumentNotFoundError, GroupNotFoundError)):
        return "404 Not Found"
    return "500 Internal Server Error"


# ── FE shape transforms ─────────────────────────────────────────────────────


def get_groups_for_doc(groups, codexis_id):
    """Return list of {id, name} for groups containing this codexisId."""
    return [
        {"id": g.get("id", ""), "name": g.get("name", "")}
        for g in groups
        if codexis_id in g.get("members", [])
    ]


def resolve_tracking_type(s):
    has_parts = bool(s.get("parts"))
    rt = s.get("related_tracking", {})
    has_related = rt.get("enabled", False)
    if has_related:
        return "all"
    if has_parts:
        return "document_changes"
    return "all"


def state_to_fe_document(s):
    """Reshape internal state into the FE-expected document shape."""
    codexis_id = s.get("codexisId", "")
    fe_changes = []
    for change in s.get("changes", []):
        if not isinstance(change, dict):
            continue
        if change.get("change_type") == "related_change":
            # related_change is a CODEXIS curation event — no legal "effective"
            # date exists for the event itself. Omit effective_on so FE can
            # render just detected_on.
            fe_change = {
                "source_documents": change.get("source_documents", []),
                "detected_on": change.get("detected_on", ""),
                "change_type": "related_change",
                "description_md": change.get("description_md", ""),
                "confirmed_on": change.get("confirmed_on"),
                "amendments": [],
            }
        else:
            fe_change = {
                "source_documents": [{
                    "codexisId": "cdx://doc/" + codexis_id,
                    "name": s.get("name", ""),
                }],
                "detected_on": change.get("detected_on", ""),
                "effective_on": change.get("effective_on", ""),
                "change_type": "document_change",
                "description_md": change.get("description_md", ""),
                "confirmed_on": change.get("confirmed_on"),
                "amendments": [
                    a if isinstance(a, dict) else {"id": a, "name": a}
                    for a in change.get("amendments", [])
                ],
            }
        if change.get("compare_url"):
            fe_change["compare_url"] = change["compare_url"]
        fe_changes.append(fe_change)

    return {
        "uuid": s.get("uuid", ""),
        "codexisId": "cdx://doc/" + codexis_id,
        "name": s.get("name", ""),
        "added_on": s.get("added_on", ""),
        "tracking_type": resolve_tracking_type(s),
        "parts": [{"partId": p, "label": p} for p in (s.get("parts") or [])],
        "changes": fe_changes,
        "user_notes": s.get("user_notes", []),
    }


def count_changes(document):
    changes = document.get("changes", [])
    total = len([c for c in changes if isinstance(c, dict)])
    unconfirmed = sum(
        1 for c in changes if isinstance(c, dict) and not c.get("confirmed_on")
    )
    return total, unconfirmed


def to_overview_entry(document, groups):
    total, unconfirmed = count_changes(document)
    raw_id = document.get("codexisId", "").replace("cdx://doc/", "")
    return {
        "uuid": document.get("uuid"),
        "codexisId": document.get("codexisId"),
        "name": document.get("name"),
        "added_on": document.get("added_on"),
        "tracking_type": document.get("tracking_type"),
        "unconfirmed_changes": unconfirmed,
        "total_changes": total,
        "groups": get_groups_for_doc(groups, raw_id),
    }


# ── watched folders ─────────────────────────────────────────────────────────


def _spawn_cli(cli_args):
    """Run cdx-sledovane-dokumenty detached so slow harvest/check don't block."""
    import subprocess
    try:
        subprocess.Popen(
            ["cdx-sledovane-dokumenty"] + cli_args,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            stdin=subprocess.DEVNULL,
            start_new_session=True,
        )
        return True
    except OSError:
        return False


def folder_to_fe(root):
    """Reshape one watched folder (watched.json + tracking) for the FE."""
    abs_root = os.path.abspath(os.path.expanduser(root))
    entry = folders.find_in_index(abs_root) or {
        "root": abs_root,
        "name": os.path.basename(abs_root.rstrip("/")) or abs_root,
        "added_on": None,
    }
    data = folders.read_watched(abs_root) or folders.empty_watched()
    tracking_state = folders.read_tracking(abs_root) or {"legislation": {}, "lastCheckAt": None}
    leg_state = tracking_state.get("legislation", {})

    documents = []
    pending = 0
    for d in data.get("documents", []):
        if d.get("extractedAt") is None:
            pending += 1
        documents.append({
            "path": d.get("path"),
            "sha256": d.get("sha256"),
            "discoveredAt": d.get("discoveredAt"),
            "extractedAt": d.get("extractedAt"),
            "updatedAt": d.get("updatedAt"),
            "legislation": d.get("legislation", []),
        })

    legislation = []
    for ref in folders.unique_legislation(abs_root):
        cid = ref.get("codexisId")
        st = leg_state.get(cid, {}) if cid else {}
        changes = st.get("changes", [])
        legislation.append({
            "uri": ref.get("uri"),
            "text": ref.get("text"),
            "codexisId": cid,
            "tracked": bool(cid),
            "baselineVersionId": st.get("baselineVersionId"),
            "lastKnownVersionId": st.get("lastKnownVersionId"),
            "changes": changes,
            "unconfirmed_changes": sum(1 for c in changes if not c.get("confirmed_on")),
        })

    return {
        "root": abs_root,
        "name": entry.get("name"),
        "added_on": entry.get("added_on"),
        "documents": documents,
        "documents_count": len(documents),
        "pending_harvest": pending,
        "legislation": legislation,
        "last_check_at": tracking_state.get("lastCheckAt"),
    }


def handle_folder_get(view, params):
    if view == "folders":
        emit_json({
            "mode": "folders",
            "generated_at": now_utc(),
            "folders": folders.list_watches(),
            "settings": notify.load_settings(),
        })
        return
    if view == "folder":
        root = (params.get("root", [""])[0] or "").strip()
        if not root:
            emit_json({"ok": False, "error": "'root' is required"}, status="400 Bad Request")
            return
        emit_json({"mode": "folder", "generated_at": now_utc(), "folder": folder_to_fe(root)})
        return
    if view == "browse":
        path = (params.get("path", [""])[0] or "").strip() or None
        emit_json({"mode": "browse", "generated_at": now_utc(), **folders.browse(path)})
        return
    if view == "notify":
        emit_json({"mode": "notify", "settings": notify.load_settings()})
        return
    emit_json({"ok": False, "error": f"unknown view: {view}"}, status="400 Bad Request")


def handle_folder_action(action, body):
    root = (body.get("root") or "").strip()

    if action == "folder_add":
        if not root:
            emit_json({"ok": False, "error": "'root' is required"}, status="400 Bad Request")
            return
        res = folders.add_watch(root, name=(body.get("name") or None))
        _spawn_cli(["folder", "harvest", res["root"]])
        emit_json({"ok": True, "folder": res, "extracting": True})
        return

    if action in ("start_extraction", "folder_harvest"):
        if not root:
            emit_json({"ok": False, "error": "'root' is required"}, status="400 Bad Request")
            return
        _spawn_cli(["folder", "harvest", os.path.abspath(os.path.expanduser(root))])
        emit_json({"ok": True, "started": True})
        return

    if action == "folder_refresh":
        if not root:
            emit_json({"ok": False, "error": "'root' is required"}, status="400 Bad Request")
            return
        _spawn_cli(["folder", "refresh", os.path.abspath(os.path.expanduser(root))])
        emit_json({"ok": True, "started": True})
        return

    if action == "folder_check":
        args = ["folder", "check"]
        if root:
            args.append(os.path.abspath(os.path.expanduser(root)))
        _spawn_cli(args)
        emit_json({"ok": True, "started": True})
        return

    if action == "folder_remove":
        if not root:
            emit_json({"ok": False, "error": "'root' is required"}, status="400 Bad Request")
            return
        folders.remove_watch(root, purge=bool(body.get("purge")))
        emit_json({"ok": True})
        return

    if action == "folder_confirm":
        if not root:
            emit_json({"ok": False, "error": "'root' is required"}, status="400 Bad Request")
            return
        marked = folder_check.confirm_folder(root, body.get("codexisId"))
        emit_json({"ok": True, "confirmed": marked})
        return

    if action == "save_notify_settings":
        saved = notify.save_settings({
            "email": bool(body.get("email", True)),
            "inApp": bool(body.get("inApp", True)),
            "recipients": body.get("recipients", []),
        })
        emit_json({"ok": True, "settings": saved})
        return

    emit_json({"ok": False, "error": f"unknown folder action: {action}"}, status="400 Bad Request")


# ── POST dispatch ───────────────────────────────────────────────────────────


def handle_post(body):
    action = body.get("action", "")

    if action in ("group_add", "group_remove", "group_delete", "group_rename"):
        handle_group_action(action, body)
        return

    if action.startswith("folder_") or action in ("start_extraction", "save_notify_settings"):
        handle_folder_action(action, body)
        return

    target_uuid = body.get("uuid", "")
    if not action or not target_uuid:
        emit_json(
            {"ok": False, "error": "'action' and 'uuid' are required"},
            status="400 Bad Request",
        )
        return

    codexis_id, s = tracking.find_by_uuid(target_uuid)
    if codexis_id is None:
        emit_json(
            {"ok": False, "error": "tracked document not found"},
            status="404 Not Found",
        )
        return

    if action == "note_add":
        text = (body.get("text") or "").strip()
        if not text:
            emit_json({"ok": False, "error": "'text' is required"}, status="400 Bad Request")
            return
        tracking.note_add(codexis_id, text)
        emit_json({"ok": True})
        return

    if action == "note_remove":
        index = body.get("index")
        if not isinstance(index, int):
            emit_json({"ok": False, "error": "'index' is required"}, status="400 Bad Request")
            return
        try:
            tracking.note_remove(codexis_id, index)
        except IndexError:
            emit_json({"ok": False, "error": "invalid index"}, status="400 Bad Request")
            return
        emit_json({"ok": True})
        return

    if action == "confirm":
        change_index = body.get("changeIndex")
        try:
            tracking.confirm(codexis_id, change_index=change_index)
        except IndexError:
            emit_json(
                {"ok": False, "error": "invalid changeIndex"}, status="400 Bad Request"
            )
            return
        emit_json({"ok": True})
        return

    if action == "remove":
        tracking.remove(codexis_id)
        emit_json({"ok": True})
        return

    emit_json({"ok": False, "error": f"unknown action: {action}"}, status="400 Bad Request")


def handle_group_action(action, body):
    if action == "group_add":
        codexis_id = (body.get("codexisId") or "").replace("cdx://doc/", "")
        group_name = (body.get("groupName") or "").strip()
        if not group_name:
            emit_json({"ok": False, "error": "'groupName' is required"}, status="400 Bad Request")
            return
        if codexis_id:
            try:
                tracking.group_add(codexis_id, group_name)
            except DocumentNotTrackedError:
                emit_json({"ok": False, "error": "tracked document not found"}, status="404 Not Found")
                return
        else:
            # Create empty group — core doesn't have this, do it inline.
            groups = state.load_groups()
            _, existing = state.find_group(groups, group_name)
            if existing is None:
                groups.append({
                    "id": state.slugify(group_name),
                    "name": group_name,
                    "members": [],
                })
                state.save_groups(groups)
        emit_json({"ok": True})
        return

    if action == "group_remove":
        codexis_id = (body.get("codexisId") or "").replace("cdx://doc/", "")
        group_id = body.get("groupId", "")
        if not codexis_id or not group_id:
            emit_json(
                {"ok": False, "error": "'codexisId' and 'groupId' are required"},
                status="400 Bad Request",
            )
            return
        tracking.group_remove_by_id(codexis_id, group_id)
        emit_json({"ok": True})
        return

    if action == "group_delete":
        group_id = body.get("groupId", "")
        if not group_id:
            emit_json({"ok": False, "error": "'groupId' is required"}, status="400 Bad Request")
            return
        tracking.group_delete_by_id(group_id)
        emit_json({"ok": True})
        return

    if action == "group_rename":
        group_id = body.get("groupId", "")
        new_name = (body.get("newName") or "").strip()
        if not group_id or not new_name:
            emit_json(
                {"ok": False, "error": "'groupId' and 'newName' are required"},
                status="400 Bad Request",
            )
            return
        tracking.group_rename(group_id, new_name)
        emit_json({"ok": True})
        return

    emit_json({"ok": False, "error": f"unknown group action: {action}"}, status="400 Bad Request")


# ── GET dispatch ────────────────────────────────────────────────────────────


def handle_get(query_string):
    params = urllib.parse.parse_qs(query_string, keep_blank_values=True)

    view = (params.get("view", [""])[0] or "").strip()
    if view:
        handle_folder_get(view, params)
        return

    requested_uuid = (params.get("uuid", [""])[0] or "").strip()

    documents = [state_to_fe_document(s) for s in tracking.list_all()]
    groups = state.load_groups()

    if requested_uuid:
        matching = next(
            (d for d in documents if str(d.get("uuid", "")).strip() == requested_uuid),
            None,
        )
        if matching is None:
            emit_json(
                {
                    "mode": "detail",
                    "generated_at": now_utc(),
                    "uuid": requested_uuid,
                    "error": "tracked document not found",
                },
                status="404 Not Found",
            )
            return

        total, unconfirmed = count_changes(matching)
        detail = dict(matching)
        detail["total_changes"] = total
        detail["unconfirmed_changes"] = unconfirmed
        raw_id = matching.get("codexisId", "").replace("cdx://doc/", "")
        detail["groups"] = get_groups_for_doc(groups, raw_id)
        emit_json({
            "mode": "detail",
            "generated_at": now_utc(),
            "document": detail,
            "groups": groups,
        })
        return

    overview_items = [to_overview_entry(d, groups) for d in documents]
    emit_json({
        "mode": "overview",
        "generated_at": now_utc(),
        "tracked_documents": overview_items,
        "groups": groups,
    })


# ── entrypoint ──────────────────────────────────────────────────────────────


def main():
    method = os.environ.get("REQUEST_METHOD", "GET").upper()
    query_string = os.environ.get("QUERY_STRING", "")

    try:
        if method == "POST":
            content_length = int(os.environ.get("CONTENT_LENGTH") or 0)
            raw = sys.stdin.read(content_length) if content_length > 0 else ""
            try:
                body = json.loads(raw) if raw else {}
            except json.JSONDecodeError:
                emit_json(
                    {"ok": False, "error": "invalid JSON body"},
                    status="400 Bad Request",
                )
                return
            handle_post(body)
        else:
            handle_get(query_string)
    except DocumentError as e:
        emit_json({"ok": False, "error": str(e)}, status=error_status(e))
    except Exception as e:  # noqa: BLE001 — last-resort guard
        emit_json(
            {"ok": False, "error": f"Internal error: {e}"},
            status="500 Internal Server Error",
        )


if __name__ == "__main__":
    main()
