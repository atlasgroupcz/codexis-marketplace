#!/usr/bin/env python3
"""JSON CGI handler for the sledovana-judikatura plugin component.

GET returns JSON data; POST dispatches JSON actions. HTML shell
(index.html) is served by the sibling `judikatura` bash wrapper's
fast path; this module only runs for JSON requests.

State lives in $CDX_USER_HOME/.cdx/apps/sledovana-judikatura/ (managed by
sledovana_judikatura_core).
"""

import json
import os
import re
import sys
import urllib.parse
from pathlib import Path

HERE = Path(__file__).resolve().parent
PLUGIN_DIR = HERE.parent.parent
LIB_DIR = PLUGIN_DIR / "lib"

sys.path.insert(0, str(LIB_DIR))

try:
    from sledovana_judikatura_core import state, tracking
    from sledovana_judikatura_core.exceptions import (
        NoteIndexError,
        ReportNotFoundError,
        TopicError,
        TopicNotFoundError,
    )
except ImportError as e:
    sys.stdout.write("Status: 500 Internal Server Error\r\n")
    sys.stdout.write("Content-Type: application/json; charset=utf-8\r\n\r\n")
    sys.stdout.write(json.dumps({"error": f"sledovana_judikatura_core import failed: {e}"}))
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


SAFE_ID_RE = re.compile(r"^[a-zA-Z0-9_-]+$")


def validate_id(value):
    """Reject path-traversal attempts. Emits 400 + exits on failure."""
    if not value or not SAFE_ID_RE.match(value):
        emit_json({"error": f"Invalid identifier: {value}"}, status="400 Bad Request")
        sys.exit(0)


# ── POST dispatch ───────────────────────────────────────────────────────────


def handle_post(body):
    action = body.get("action", "")

    if action == "confirm_report":
        topic_uuid = body.get("uuid")
        report_id = body.get("report_id")
        if not topic_uuid or not report_id:
            emit_json(
                {"error": "uuid and report_id required"},
                status="400 Bad Request",
            )
            return
        validate_id(topic_uuid)
        validate_id(report_id)
        try:
            confirmed_on = tracking.confirm_report(topic_uuid, report_id)
        except (TopicNotFoundError, ReportNotFoundError):
            emit_json({"error": "Report not found"}, status="404 Not Found")
            return
        emit_json({"ok": True, "confirmed_on": confirmed_on})
        return

    if action == "delete":
        topic_uuid = body.get("uuid")
        if not topic_uuid:
            emit_json({"error": "uuid required"}, status="400 Bad Request")
            return
        validate_id(topic_uuid)
        try:
            tracking.delete(topic_uuid)
        except TopicNotFoundError:
            emit_json({"error": "Topic not found"}, status="404 Not Found")
            return
        emit_json({"ok": True})
        return

    if action == "note_add":
        topic_uuid = body.get("uuid")
        text = (body.get("text") or "").strip()
        if not topic_uuid or not text:
            emit_json({"error": "uuid and text required"}, status="400 Bad Request")
            return
        validate_id(topic_uuid)
        try:
            tracking.note_add(topic_uuid, text)
        except TopicNotFoundError:
            emit_json({"error": "Topic not found"}, status="404 Not Found")
            return
        emit_json({"ok": True})
        return

    if action == "note_remove":
        topic_uuid = body.get("uuid")
        index = body.get("index")
        if not topic_uuid or not isinstance(index, int):
            emit_json(
                {"error": "uuid and index (int) required"},
                status="400 Bad Request",
            )
            return
        validate_id(topic_uuid)
        try:
            tracking.note_remove(topic_uuid, index)
        except TopicNotFoundError:
            emit_json({"error": "Topic not found"}, status="404 Not Found")
            return
        except NoteIndexError as e:
            emit_json({"error": str(e)}, status="400 Bad Request")
            return
        emit_json({"ok": True})
        return

    emit_json({"error": f"Unknown action: {action}"}, status="400 Bad Request")


# ── GET dispatch ────────────────────────────────────────────────────────────


def handle_get(query_string):
    params = urllib.parse.parse_qs(query_string)
    topic_uuid = params.get("uuid", [None])[0]
    report_id = params.get("report", [None])[0]
    if topic_uuid:
        validate_id(topic_uuid)
    if report_id:
        validate_id(report_id)

    if topic_uuid and report_id:
        report = state.load_report(topic_uuid, report_id)
        if report is None:
            emit_json({"error": "Report not found"}, status="404 Not Found")
            return
        topic = state.load_topic(topic_uuid) or {}
        emit_json({
            "mode": "report",
            "generated_at": now_utc(),
            "topic": {"uuid": topic_uuid, "name": topic.get("name", "")},
            "report": report,
        })
        return

    if topic_uuid:
        topic = state.load_topic(topic_uuid)
        if topic is None:
            emit_json({"error": "Topic not found"}, status="404 Not Found")
            return
        emit_json({
            "mode": "detail",
            "generated_at": now_utc(),
            "topic": topic,
            "reports": state.report_summaries(topic_uuid),
        })
        return

    overview = []
    for t in tracking.list_all():
        overview.append({
            "uuid": t["uuid"],
            "name": t["name"],
            "areas": len(t["areas"]),
            "notes": len(t["notes"]),
            "created_at": t["created_at"],
            "last_check_at": t["last_check_at"],
            "total_reports": t["total_reports"],
            "unconfirmed_reports": t["unconfirmed_reports"],
        })
    emit_json({
        "mode": "overview",
        "generated_at": now_utc(),
        "topics": overview,
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
                emit_json({"error": "Invalid JSON body"}, status="400 Bad Request")
                return
            handle_post(body)
        else:
            handle_get(query_string)
    except TopicError as e:
        emit_json({"error": str(e)}, status="500 Internal Server Error")
    except Exception as e:  # noqa: BLE001 — last-resort guard
        emit_json(
            {"error": f"Internal error: {e}"},
            status="500 Internal Server Error",
        )


if __name__ == "__main__":
    main()
