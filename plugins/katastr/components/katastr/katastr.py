#!/usr/bin/env python3
"""JSON CGI handler for the katastr plugin component.

GET returns JSON data; POST dispatches JSON actions. HTML shell
(index.html) is served by the sibling `katastr` bash wrapper's
fast path; this module only runs for JSON requests.

State lives in $CDX_USER_HOME/.cdx/apps/katastr/ (managed by katastr_core).
"""

import json
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
    from katastr_core import settings, tracking
    from katastr_core.exceptions import (
        ApiHttpError,
        ApiKeyInvalidError,
        ApiKeyMissingError,
        ApiNetworkError,
        InvalidProceedingNumberError,
        KatastrError,
        ProceedingAlreadyTrackedError,
        ProceedingNotFoundError,
        ProceedingNotTrackedError,
    )
except ImportError as e:
    sys.stdout.write("Status: 500 Internal Server Error\r\n")
    sys.stdout.write("Content-Type: application/json; charset=utf-8\r\n\r\n")
    sys.stdout.write(json.dumps({"error": f"katastr_core import failed: {e}"}))
    sys.exit(0)


# ── output helpers ──────────────────────────────────────────────────────────


def emit_json(payload, status: Optional[str] = None) -> None:
    if status:
        sys.stdout.write(f"Status: {status}\r\n")
    sys.stdout.write("Content-Type: application/json; charset=utf-8\r\n")
    sys.stdout.write("Cache-Control: no-store\r\n")
    sys.stdout.write("\r\n")
    sys.stdout.write(json.dumps(payload, indent=2, ensure_ascii=False))


def error_status(exc: Exception) -> str:
    """Map exception type to HTTP status."""
    if isinstance(exc, ProceedingAlreadyTrackedError):
        return "409 Conflict"
    if isinstance(exc, (ProceedingNotTrackedError, ProceedingNotFoundError)):
        return "404 Not Found"
    if isinstance(exc, InvalidProceedingNumberError):
        return "400 Bad Request"
    if isinstance(exc, ApiKeyMissingError):
        return "412 Precondition Failed"
    if isinstance(exc, ApiKeyInvalidError):
        return "401 Unauthorized"
    if isinstance(exc, ApiNetworkError):
        return "503 Service Unavailable"
    if isinstance(exc, ApiHttpError):
        return f"{exc.status} ČÚZK Error"
    return "500 Internal Server Error"


# ── action helpers ──────────────────────────────────────────────────────────


def with_proceeding_uuid(body: dict, fn: Callable[[dict], None]) -> None:
    """Look up a proceeding by uuid from body and call fn(state). Emits 400/404 if missing."""
    proc_uuid = body.get("uuid")
    if not proc_uuid:
        emit_json({"ok": False, "error": "uuid required"}, status="400 Bad Request")
        return
    state = tracking.find_by_uuid(proc_uuid)
    if state is None:
        emit_json(
            {"ok": False, "error": "Proceeding not found"},
            status="404 Not Found",
        )
        return
    fn(state)


# ── action dispatch ─────────────────────────────────────────────────────────


def handle_post(body: dict) -> None:
    action = body.get("action", "")

    if action == "save_api_key":
        api_key = (body.get("apiKey") or "").strip()
        if not api_key:
            emit_json(
                {"ok": False, "error": "API klíč je prázdný."},
                status="400 Bad Request",
            )
            return
        settings.set_api_key(api_key)
        emit_json({"ok": True, "saved": True})

    elif action == "delete_api_key":
        settings.write_api_key("")
        emit_json({"ok": True})

    elif action == "add_proceeding":
        cislo = (body.get("cislo_rizeni") or "").strip()
        label = (body.get("label") or "").strip()
        state = tracking.add(cislo, label=label)
        emit_json({"ok": True, "proceeding": tracking.state_to_overview_entry(state)})

    elif action == "remove":
        def _remove(state):
            tracking.remove(state["cislo_rizeni"])
            emit_json({"ok": True})
        with_proceeding_uuid(body, _remove)

    elif action == "check_all":
        results = tracking.check_all()
        new_changes = sum(1 for r in results if r.get("ok") and r.get("change"))
        errors = [
            {"cislo_rizeni": r["cislo_rizeni"], "error": r["error"]}
            for r in results
            if not r.get("ok")
        ]
        emit_json(
            {
                "ok": True,
                "checked": len(results),
                "new_changes": new_changes,
                "errors": errors,
            }
        )

    elif action == "confirm_change":
        def _confirm(state):
            change_index = body.get("change_index")
            marked = tracking.confirm(
                state["cislo_rizeni"],
                change_index=change_index if isinstance(change_index, int) else None,
            )
            emit_json({"ok": True, "marked": marked})
        with_proceeding_uuid(body, _confirm)

    elif action == "set_label":
        def _set_label(state):
            tracking.set_label(state["cislo_rizeni"], body.get("label", ""))
            emit_json({"ok": True})
        with_proceeding_uuid(body, _set_label)

    else:
        emit_json(
            {"ok": False, "error": f"Unknown action: {action}"},
            status="400 Bad Request",
        )


def handle_get(query_string: str) -> None:
    params = urllib.parse.parse_qs(query_string, keep_blank_values=True)

    if "settings" in params:
        if settings.is_configured():
            emit_json(
                {
                    "configured": True,
                    "maskedKey": settings.mask_key(settings.read_api_key()),
                }
            )
        else:
            emit_json({"configured": False})
        return

    requested_uuid = (params.get("uuid", [None])[0] or "").strip()
    if requested_uuid:
        state = tracking.find_by_uuid(requested_uuid)
        if state is None:
            emit_json({"error": "Proceeding not found"}, status="404 Not Found")
            return
        detail = tracking.state_to_overview_entry(state)
        detail["changes"] = state.get("changes", [])
        emit_json(
            {
                "mode": "detail",
                "generated_at": tracking.now_utc(),
                "proceeding": detail,
            }
        )
        return

    states = tracking.list_all()
    items = [tracking.state_to_overview_entry(s) for s in states]
    emit_json(
        {
            "mode": "overview",
            "generated_at": tracking.now_utc(),
            "proceedings": items,
            "api_key_configured": settings.is_configured(),
            "api_key_masked": (
                settings.mask_key(settings.read_api_key())
                if settings.is_configured()
                else ""
            ),
        }
    )


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
                emit_json(
                    {"ok": False, "error": "Invalid JSON body"},
                    status="400 Bad Request",
                )
                return
            handle_post(body)
        else:
            handle_get(query_string)
    except KatastrError as e:
        emit_json({"ok": False, "error": str(e)}, status=error_status(e))
    except Exception as e:  # noqa: BLE001
        emit_json(
            {"ok": False, "error": f"Internal error: {e}"},
            status="500 Internal Server Error",
        )


if __name__ == "__main__":
    main()
