#!/usr/bin/env python3
"""JSON CGI handler for the ONEPOST plugin component."""

import json
import os
import sys
from pathlib import Path
from typing import Optional

HERE = Path(__file__).resolve().parent
PLUGIN_DIR = HERE.parent.parent
LIB_DIR = PLUGIN_DIR / "lib"
sys.path.insert(0, str(LIB_DIR))

try:
    from onepost_core import settings
    from onepost_core.api_client import OnepostClient
    from onepost_core.exceptions import (
        ApiAuthError,
        ApiHttpError,
        ApiKeyMissingError,
        ApiNetworkError,
        ApiPermissionError,
        ApiRateLimitError,
        ConfigError,
        OnepostError,
    )
except ImportError as e:
    sys.stdout.write("Status: 500 Internal Server Error\r\n")
    sys.stdout.write("Content-Type: application/json; charset=utf-8\r\n\r\n")
    sys.stdout.write(json.dumps({"ok": False, "error": f"onepost_core import failed: {e}"}))
    sys.exit(0)


def emit_json(payload, status: Optional[str] = None) -> None:
    if status:
        sys.stdout.write(f"Status: {status}\r\n")
    sys.stdout.write("Content-Type: application/json; charset=utf-8\r\n")
    sys.stdout.write("Cache-Control: no-store\r\n")
    sys.stdout.write("\r\n")
    sys.stdout.write(json.dumps(payload, indent=2, ensure_ascii=False))


def error_status(exc: Exception) -> str:
    if isinstance(exc, ConfigError):
        return "400 Bad Request"
    if isinstance(exc, ApiKeyMissingError):
        return "412 Precondition Failed"
    if isinstance(exc, ApiAuthError):
        return "401 Unauthorized"
    if isinstance(exc, ApiPermissionError):
        return "403 Forbidden"
    if isinstance(exc, ApiRateLimitError):
        return "429 Too Many Requests"
    if isinstance(exc, ApiNetworkError):
        return "503 Service Unavailable"
    if isinstance(exc, ApiHttpError):
        return f"{exc.status} ONEPOST Error"
    return "500 Internal Server Error"


def settings_payload() -> dict:
    api_key = settings.read_api_key()
    api_key_url = settings.read_api_key_url()
    return {
        "configured": bool(api_key),
        "maskedKey": settings.mask_key(api_key) if api_key else "",
        "keySource": settings.read_api_key_source(),
        "apiUrl": settings.read_api_url(),
        "settingsFile": settings.ENV_FILE,
        "generateUrl": api_key_url,
    }


def handle_post(body: dict) -> None:
    action = body.get("action", "")

    if action == "save_api_key":
        api_key = (body.get("apiKey") or "").strip()
        if not api_key:
            emit_json({"ok": False, "error": "API klíč je prázdný."}, status="400 Bad Request")
            return
        mailboxes = OnepostClient(api_key=api_key).get_mailboxes()
        settings.write_api_key(api_key)
        emit_json(
            {
                "ok": True,
                "saved": True,
                "maskedKey": settings.mask_key(api_key),
                "mailboxCount": len(mailboxes) if isinstance(mailboxes, list) else None,
            }
        )
        return

    if action == "test_api_key":
        mailboxes = OnepostClient().get_mailboxes()
        emit_json({"ok": True, "mailboxCount": len(mailboxes) if isinstance(mailboxes, list) else None})
        return

    if action == "delete_api_key":
        settings.write_api_key("")
        emit_json({"ok": True})
        return

    emit_json({"ok": False, "error": f"Unknown action: {action}"}, status="400 Bad Request")


def handle_get(_query_string: str) -> None:
    emit_json(settings_payload())


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
            return
        handle_get(query_string)
    except OnepostError as e:
        emit_json({"ok": False, "error": str(e)}, status=error_status(e))
    except Exception as e:
        emit_json({"ok": False, "error": f"Internal error: {e}"}, status="500 Internal Server Error")


if __name__ == "__main__":
    main()
