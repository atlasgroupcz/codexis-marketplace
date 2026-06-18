"""Notify the user (from the daemon) when watched-folder legislation changes.

Two daemon-native, user-configurable channels:
- e-mail  → POST /rest/v1/plugin/email/send (recipient defaults to the user)
- in-app  → GraphQL createNotification (the notification-center bell)

Settings live in ``<APP_DIR>/folder-watch-settings.json`` and are edited from the
app's settings UI. Both channels are best-effort: a missing token or a transport
error is swallowed so a scheduled check never fails because of notifications.
"""

import json
import os
import urllib.error
import urllib.request
from html import escape
from uuid import uuid4

from . import folders, state

_EMAIL_ENDPOINT = "/rest/v1/plugin/email/send"
_GRAPHQL_PATH = "/graphql"

DEFAULT_SETTINGS = {"email": True, "inApp": True, "recipients": []}

# Deep link into the Sledované dokumenty app (folders tab).
APP_LINK = "/sledovane-dokumenty?tab=folders"


# ── settings ─────────────────────────────────────────────────────────────────


def load_settings():
    path = folders.settings_path()
    if not os.path.isfile(path):
        return dict(DEFAULT_SETTINGS)
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
    except (json.JSONDecodeError, OSError):
        return dict(DEFAULT_SETTINGS)
    merged = dict(DEFAULT_SETTINGS)
    if isinstance(data, dict):
        merged.update({k: data[k] for k in DEFAULT_SETTINGS if k in data})
    return merged


def save_settings(settings):
    merged = dict(DEFAULT_SETTINGS)
    merged.update({k: settings[k] for k in DEFAULT_SETTINGS if k in settings})
    state.atomic_write_json(folders.settings_path(), merged)
    return merged


# ── transport ────────────────────────────────────────────────────────────────


def _daemon_url():
    return os.environ.get("CODEXIS_PUBLIC_DAEMON_URL")


def _token():
    return os.environ.get("CODEXIS_USER_API_TOKEN")


def _send_email(subject, body_text, body_html, recipients=None):
    daemon_url, token = _daemon_url(), _token()
    if not daemon_url or not token:
        return False

    payload = {"subject": subject, "bodyText": body_text}
    if body_html:
        payload["bodyHtml"] = body_html
    if recipients:
        payload["to"] = list(recipients)

    boundary = "----sledovane-" + uuid4().hex
    body = (
        f"--{boundary}\r\n"
        'Content-Disposition: form-data; name="payload"\r\n'
        "Content-Type: application/json; charset=utf-8\r\n\r\n"
        f"{json.dumps(payload, ensure_ascii=False)}\r\n"
        f"--{boundary}--\r\n"
    ).encode("utf-8")
    req = urllib.request.Request(
        daemon_url.rstrip("/") + _EMAIL_ENDPOINT,
        data=body,
        method="POST",
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": f"multipart/form-data; boundary={boundary}",
        },
    )
    try:
        urllib.request.urlopen(req, timeout=30).close()
        return True
    except (urllib.error.URLError, OSError):
        return False


def _graphql_url():
    base = (_daemon_url() or "").rstrip("/")
    if base.endswith(_GRAPHQL_PATH):
        return base
    return base + _GRAPHQL_PATH


def _send_inapp(message, *, action=None, link=None, ntype="legislation-changed"):
    daemon_url, token = _daemon_url(), _token()
    if not daemon_url or not token:
        return False

    mutation = (
        "mutation Create($input: CreateNotificationInput!) "
        "{ createNotification(input: $input) { id } }"
    )
    notification_input = {"message": message, "type": ntype}
    if action:
        notification_input["action"] = action
    if link:
        notification_input["link"] = link
    body = json.dumps(
        {"query": mutation, "variables": {"input": notification_input}},
        ensure_ascii=False,
    ).encode("utf-8")
    req = urllib.request.Request(
        _graphql_url(),
        data=body,
        method="POST",
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json; charset=utf-8",
        },
    )
    try:
        urllib.request.urlopen(req, timeout=30).close()
        return True
    except (urllib.error.URLError, OSError):
        return False


# ── summaries ────────────────────────────────────────────────────────────────


def _changed_folders(results):
    return [r for r in results if r.get("changes")]


def build_summary(results):
    """Build (subject, body_text, body_html) for folders with new changes, or None."""
    changed = _changed_folders(results)
    total = sum(len(r["changes"]) for r in changed)
    if total == 0:
        return None

    if total == 1:
        subject = "Sledované dokumenty: změna v odkazované legislativě"
    else:
        subject = f"Sledované dokumenty: {total} změn v odkazované legislativě"

    text_lines = ["Kontrola sledovaných složek zaznamenala změny:", ""]
    html_rows = []
    for r in changed:
        text_lines.append(f"Složka „{r['name']}“ — {len(r['changes'])} změn:")
        for c in r["changes"]:
            eff = f" (účinné od {c['effective_on']})" if c.get("effective_on") else ""
            text_lines.append(f"  • {c.get('text') or c.get('codexisId')}{eff}")
        text_lines.append("")
        items = "".join(
            f'<li style="margin:2px 0">{escape(c.get("text") or c.get("codexisId") or "")}'
            + (f' <span style="color:#777">(účinné od {escape(c["effective_on"])})</span>'
               if c.get("effective_on") else "")
            + "</li>"
            for c in r["changes"]
        )
        html_rows.append(
            f'<h3 style="margin:18px 0 6px;font:600 15px sans-serif;color:#0a0a0a">'
            f"Složka „{escape(r['name'])}“</h3>"
            f'<ul style="margin:0;padding-left:18px;font:14px/1.5 sans-serif;color:#333">{items}</ul>'
        )
    text_lines.append("Otevřete aplikaci Sledované dokumenty v CODEXIS pro detail.")

    body_html = (
        '<div style="max-width:560px;font:14px sans-serif;color:#0a0a0a">'
        '<p style="margin:0;font:600 12px sans-serif;letter-spacing:.14em;'
        'text-transform:uppercase;color:#5ea500">Sledované dokumenty</p>'
        '<h2 style="margin:4px 0 2px;font:400 22px Georgia,serif">'
        "Změny v odkazované legislativě</h2>"
        + "".join(html_rows)
        + '<p style="margin-top:20px;font:13px sans-serif;color:#777">'
        "Otevřete aplikaci Sledované dokumenty v CODEXIS pro detail.</p></div>"
    )
    return subject, "\n".join(text_lines), body_html


def notify_changes(results, settings=None):
    """Send e-mail and/or in-app notifications for detected changes per settings.

    Returns {"email": bool, "inApp": bool, "changes": int}. Best-effort; never
    raises on transport failure.
    """
    summary = build_summary(results)
    total = sum(len(r["changes"]) for r in _changed_folders(results))
    if summary is None:
        return {"email": False, "inApp": False, "changes": 0}

    settings = settings or load_settings()
    subject, body_text, body_html = summary

    email_sent = inapp_sent = False
    if settings.get("email"):
        email_sent = _send_email(
            subject, body_text, body_html, recipients=settings.get("recipients")
        )
    if settings.get("inApp"):
        inapp_sent = _send_inapp(
            subject, action="Zobrazit změny", link=APP_LINK
        )
    return {"email": email_sent, "inApp": inapp_sent, "changes": total}
