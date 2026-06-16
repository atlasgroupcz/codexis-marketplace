#!/usr/bin/env python3
"""JSON CGI handler for the znamky (trademark watchdog) plugin component.

GET returns JSON data (or raw logo bytes); POST dispatches JSON actions. The
HTML shell (index.html) is served by the daemon's SpaCgi runtime directly from
`entrypoint` in component.json, so this module only runs for /api requests.

State lives in $CODEXIS_PUBLIC_USER_HOME/.cdx/apps/znamky/sledovane/ (managed by
znamky_core.tracking).
"""

import base64
import json
import mimetypes
import os
import sys
import urllib.parse
from pathlib import Path
from typing import Callable, Optional

HERE = Path(__file__).resolve().parent
PLUGIN_DIR = HERE.parent.parent
LIB_DIR = PLUGIN_DIR / "lib"
sys.path.insert(0, str(LIB_DIR))

try:
    from znamky_core import settings, tracking
    from znamky_core.exceptions import (
        ApiHttpError,
        ApiNetworkError,
        ApiParseError,
        CredentialsInvalidError,
        CredentialsMissingError,
        InvalidMarkError,
        MarkAlreadyTrackedError,
        MarkNotTrackedError,
        ZnamkyError,
    )
except ImportError as e:
    sys.stdout.write("Status: 500 Internal Server Error\r\n")
    sys.stdout.write("Content-Type: application/json; charset=utf-8\r\n\r\n")
    sys.stdout.write(json.dumps({"error": f"znamky_core import failed: {e}"}))
    sys.exit(0)


# ── output helpers ──────────────────────────────────────────────────────────


def emit_json(payload, status: Optional[str] = None) -> None:
    if status:
        sys.stdout.write(f"Status: {status}\r\n")
    sys.stdout.write("Content-Type: application/json; charset=utf-8\r\n")
    sys.stdout.write("Cache-Control: no-store\r\n\r\n")
    sys.stdout.write(json.dumps(payload, indent=2, ensure_ascii=False))


def emit_image(path: str) -> None:
    if not path or not os.path.isfile(path):
        emit_json({"error": "Obrázek nenalezen"}, status="404 Not Found")
        return
    ctype = mimetypes.guess_type(path)[0] or "application/octet-stream"
    with open(path, "rb") as f:
        data = f.read()
    sys.stdout.write(f"Content-Type: {ctype}\r\n")
    sys.stdout.write("Cache-Control: private, max-age=86400\r\n")
    sys.stdout.write(f"Content-Length: {len(data)}\r\n\r\n")
    sys.stdout.flush()
    sys.stdout.buffer.write(data)


def error_status(exc: Exception) -> str:
    if isinstance(exc, MarkAlreadyTrackedError):
        return "409 Conflict"
    if isinstance(exc, MarkNotTrackedError):
        return "404 Not Found"
    if isinstance(exc, InvalidMarkError):
        return "400 Bad Request"
    if isinstance(exc, CredentialsInvalidError):
        return "401 Unauthorized"
    if isinstance(exc, CredentialsMissingError):
        return "412 Precondition Failed"
    if isinstance(exc, ApiNetworkError):
        return "503 Service Unavailable"
    if isinstance(exc, ApiParseError):
        return "502 Bad Gateway"
    if isinstance(exc, ApiHttpError):
        return f"{exc.status} Source Error"
    return "500 Internal Server Error"


def with_mark_uuid(body: dict, fn: Callable[[dict], None]) -> None:
    mark_uuid = body.get("uuid")
    if not mark_uuid:
        emit_json({"ok": False, "error": "uuid required"}, status="400 Bad Request")
        return
    state = tracking.find_by_uuid(mark_uuid)
    if state is None:
        emit_json({"ok": False, "error": "Známka nenalezena"}, status="404 Not Found")
        return
    fn(state)


# ── action dispatch ─────────────────────────────────────────────────────────


def handle_post(body: dict) -> None:
    action = body.get("action", "")

    if action == "save_config":
        result = settings.set_credentials(body.get("client_id", ""), body.get("client_secret", ""))
        emit_json({"ok": True, "verified": result["verified"], "warning": result["warning"]})
        return

    if action == "delete_config":
        settings.write_credentials("", "")
        emit_json({"ok": True})
        return

    if action == "add_text":
        state = tracking.add_text(
            body.get("text", ""),
            nice_classes=body.get("nice_classes"),
            territories=body.get("territories"),
            owner_name=body.get("owner_name", ""),
            label=body.get("label", ""),
        )
        emit_json({"ok": True, "mark": tracking.state_to_overview_entry(state)})

    elif action == "add_logo":
        raw_b64 = body.get("logo_base64") or ""
        try:
            raw = base64.b64decode(raw_b64.split(",", 1)[-1]) if raw_b64 else b""
        except Exception:  # noqa: BLE001
            emit_json({"ok": False, "error": "Neplatný base64 obrázek"}, status="400 Bad Request")
            return
        state = tracking.add_logo(
            raw, mime=body.get("mime", "image/png"),
            text=body.get("text", ""),
            nice_classes=body.get("nice_classes"),
            territories=body.get("territories"),
            vienna_codes=body.get("vienna_codes"),
            owner_name=body.get("owner_name", ""),
            label=body.get("label", ""),
        )
        emit_json({"ok": True, "mark": tracking.state_to_overview_entry(state)})

    elif action == "remove":
        def _remove(state):
            tracking.remove(state["uuid"])
            emit_json({"ok": True})
        with_mark_uuid(body, _remove)

    elif action == "check_subject":
        def _check(state):
            result = tracking.check_one(state["uuid"])
            if not result["ok"]:
                emit_json({"ok": False, "error": result["error"]}, status="503 Service Unavailable")
                return
            emit_json({"ok": True, "new_collisions": len(result["new_collisions"])})
        with_mark_uuid(body, _check)

    elif action == "check_all":
        results = tracking.check_all()
        new = sum(len(r.get("new_collisions", [])) for r in results if r.get("ok"))
        errors = [{"display_name": r["display_name"], "error": r["error"]}
                  for r in results if not r.get("ok")]
        emit_json({"ok": True, "checked": len(results), "new_collisions": new, "errors": errors})

    elif action == "confirm_change":
        def _confirm(state):
            change_index = body.get("change_index")
            marked = tracking.confirm(
                state["uuid"],
                change_index=change_index if isinstance(change_index, int) else None,
            )
            emit_json({"ok": True, "marked": marked})
        with_mark_uuid(body, _confirm)

    elif action == "set_label":
        def _set_label(state):
            tracking.set_label(state["uuid"], body.get("label", ""))
            emit_json({"ok": True})
        with_mark_uuid(body, _set_label)

    elif action == "update_mark":
        def _update(state):
            updated = tracking.update_mark(
                state["uuid"],
                text=body.get("text"),
                nice_classes=body.get("nice_classes"),
                territories=body.get("territories"),
                owner_name=body.get("owner_name"),
                label=body.get("label"),
                threshold=body.get("threshold"),
            )
            emit_json({"ok": True, "mark": tracking.state_to_overview_entry(updated)})
        with_mark_uuid(body, _update)

    elif action == "set_assessment":
        def _assess(state):
            collision = tracking.set_assessment(
                state["uuid"], int(body.get("change_index", -1)), body.get("assessment") or {}
            )
            emit_json({"ok": True, "ai_assessment": collision.get("ai_assessment")})
        with_mark_uuid(body, _assess)

    else:
        emit_json({"ok": False, "error": f"Unknown action: {action}"}, status="400 Bad Request")


def handle_get(query_string: str) -> None:
    params = urllib.parse.parse_qs(query_string, keep_blank_values=True)

    if "settings" in params:
        if settings.is_configured():
            cid, _secret = settings.read_credentials()
            emit_json({"configured": True, "maskedKey": settings.mask(cid)})
        else:
            emit_json({"configured": False})
        return

    logo_uuid = (params.get("logo", [None])[0] or "").strip()
    if logo_uuid:
        state = tracking.find_by_uuid(logo_uuid)
        emit_image(state.get("logo_path") if state else None)
        return

    cand_uuid = (params.get("candidate_logo", [None])[0] or "").strip()
    if cand_uuid:
        state = tracking.find_by_uuid(cand_uuid)
        index = params.get("index", ["-1"])[0]
        path = None
        if state and index.lstrip("-").isdigit():
            collisions = state.get("collisions", [])
            i = int(index)
            if 0 <= i < len(collisions):
                path = collisions[i].get("image_path")
        emit_image(path)
        return

    requested_uuid = (params.get("uuid", [None])[0] or "").strip()
    if requested_uuid:
        state = tracking.find_by_uuid(requested_uuid)
        if state is None:
            emit_json({"error": "Známka nenalezena"}, status="404 Not Found")
            return
        emit_json({"mode": "detail", "generated_at": tracking.now_utc(),
                   "mark": tracking.state_to_detail_entry(state)})
        return

    states = tracking.list_all()
    emit_json({"mode": "overview", "generated_at": tracking.now_utc(),
               "configured": settings.is_configured(),
               "marks": [tracking.state_to_overview_entry(s) for s in states]})


# ── entrypoint ──────────────────────────────────────────────────────────────


def main() -> None:
    method = os.environ.get("REQUEST_METHOD", "GET").upper()
    query_string = os.environ.get("QUERY_STRING", "")
    try:
        if method == "POST":
            content_length = int(os.environ.get("CONTENT_LENGTH") or 0)
            raw = sys.stdin.read(content_length) if content_length > 0 else ""
            try:
                body = json.loads(raw) if raw else {}
            except json.JSONDecodeError:
                emit_json({"ok": False, "error": "Invalid JSON body"}, status="400 Bad Request")
                return
            handle_post(body)
        else:
            handle_get(query_string)
    except ZnamkyError as e:
        emit_json({"ok": False, "error": str(e)}, status=error_status(e))
    except Exception as e:  # noqa: BLE001
        emit_json({"ok": False, "error": f"Internal error: {e}"}, status="500 Internal Server Error")


if __name__ == "__main__":
    main()
