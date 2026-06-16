#!/usr/bin/env python3
"""JSON CGI handler for the insolvence plugin component.

GET returns JSON data; POST dispatches JSON actions. The HTML shell
(index.html) is served by the daemon's SpaCgi runtime directly from
`entrypoint` in component.json, so this module only runs for JSON requests.

State lives in $CODEXIS_PUBLIC_USER_HOME/.cdx/apps/insolvence/ (managed by insolvence_core).
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
    from insolvence_core import dph, tracking
    from insolvence_core.exceptions import (
        ApiHttpError,
        ApiNetworkError,
        ApiParseError,
        InsolvenceError,
        InvalidSubjectError,
        SubjectAlreadyTrackedError,
        SubjectNotTrackedError,
    )
except ImportError as e:
    sys.stdout.write("Status: 500 Internal Server Error\r\n")
    sys.stdout.write("Content-Type: application/json; charset=utf-8\r\n\r\n")
    sys.stdout.write(json.dumps({"error": f"insolvence_core import failed: {e}"}))
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
    if isinstance(exc, SubjectAlreadyTrackedError):
        return "409 Conflict"
    if isinstance(exc, SubjectNotTrackedError):
        return "404 Not Found"
    if isinstance(exc, InvalidSubjectError):
        return "400 Bad Request"
    if isinstance(exc, ApiNetworkError):
        return "503 Service Unavailable"
    if isinstance(exc, ApiParseError):
        return "502 Bad Gateway"
    if isinstance(exc, ApiHttpError):
        return f"{exc.status} ISIR Error"
    return "500 Internal Server Error"


# ── action helpers ──────────────────────────────────────────────────────────


def with_subject_uuid(body: dict, fn: Callable[[dict], None]) -> None:
    """Look up a subject by uuid from body and call fn(state). Emits 400/404 if missing."""
    subject_uuid = body.get("uuid")
    if not subject_uuid:
        emit_json({"ok": False, "error": "uuid required"}, status="400 Bad Request")
        return
    state = tracking.find_by_uuid(subject_uuid)
    if state is None:
        emit_json(
            {"ok": False, "error": "Subjekt nenalezen"},
            status="404 Not Found",
        )
        return
    fn(state)


# ── action dispatch ─────────────────────────────────────────────────────────


def handle_post(body: dict) -> None:
    action = body.get("action", "")

    if action == "add_company":
        ico = (body.get("ico") or "").strip()
        label = (body.get("label") or "").strip()
        state = tracking.add_company(ico, label=label)
        emit_json({"ok": True, "subject": tracking.state_to_overview_entry(state)})

    elif action == "add_person":
        prijmeni = (body.get("prijmeni") or "").strip()
        jmeno = (body.get("jmeno") or "").strip()
        datum_narozeni = (body.get("datum_narozeni") or "").strip()
        label = (body.get("label") or "").strip()
        state = tracking.add_person(prijmeni, jmeno, datum_narozeni, label=label)
        emit_json({"ok": True, "subject": tracking.state_to_overview_entry(state)})

    elif action == "remove":
        def _remove(state):
            tracking.remove(state["uuid"])
            emit_json({"ok": True})
        with_subject_uuid(body, _remove)

    elif action == "check_subject":
        def _check(state):
            result = tracking.check_one(state["uuid"])
            if not result["ok"]:
                emit_json(
                    {"ok": False, "error": result["error"]},
                    status="503 Service Unavailable",
                )
                return
            emit_json({"ok": True, "new_changes": len(result["changes"])})
        with_subject_uuid(body, _check)

    elif action == "check_all":
        results = tracking.check_all()
        new_changes = sum(len(r.get("changes", [])) for r in results if r.get("ok"))
        errors = [
            {"display_name": r["display_name"], "error": r["error"]}
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
                state["uuid"],
                change_index=change_index if isinstance(change_index, int) else None,
            )
            emit_json({"ok": True, "marked": marked})
        with_subject_uuid(body, _confirm)

    elif action == "set_label":
        def _set_label(state):
            tracking.set_label(state["uuid"], body.get("label", ""))
            emit_json({"ok": True})
        with_subject_uuid(body, _set_label)

    else:
        emit_json(
            {"ok": False, "error": f"Unknown action: {action}"},
            status="400 Bad Request",
        )


def handle_get(query_string: str) -> None:
    params = urllib.parse.parse_qs(query_string, keep_blank_values=True)

    dph_dic = (params.get("dph", [None])[0] or "").strip()
    if dph_dic:
        try:
            result = dph.lookup_dph(dph_dic)
        except InsolvenceError as e:
            emit_json({"mode": "dph", "error": str(e)}, status="502 Bad Gateway")
            return
        emit_json({"mode": "dph", "generated_at": tracking.now_utc(), **result})
        return

    requested_uuid = (params.get("uuid", [None])[0] or "").strip()
    if requested_uuid:
        state = tracking.find_by_uuid(requested_uuid)
        if state is None:
            emit_json({"error": "Subjekt nenalezen"}, status="404 Not Found")
            return
        emit_json(
            {
                "mode": "detail",
                "generated_at": tracking.now_utc(),
                "subject": tracking.state_to_detail_entry(state),
            }
        )
        return

    states = tracking.list_all()
    emit_json(
        {
            "mode": "overview",
            "generated_at": tracking.now_utc(),
            "subjects": [tracking.state_to_overview_entry(s) for s in states],
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
    except InsolvenceError as e:
        emit_json({"ok": False, "error": str(e)}, status=error_status(e))
    except Exception as e:  # noqa: BLE001
        emit_json(
            {"ok": False, "error": f"Internal error: {e}"},
            status="500 Internal Server Error",
        )


if __name__ == "__main__":
    main()
