"""Notify the user (from the daemon) when watched-folder legislation changes.

Two daemon-native, user-configurable channels:
- e-mail  → POST /rest/v1/plugin/email/send (recipient defaults to the user)
- in-app  → GraphQL createNotification (the notification-center bell)

Notification settings come from the unified ``settings`` module
(``settings.get_notify()``), keyed off the app's ``settings.json`` plus the
shipped ``defaults.json``, and are edited from the app's settings UI. Both
channels are best-effort: a missing token or a transport error is swallowed
so a scheduled check never fails because of notifications.
"""

import json
import os
import urllib.error
import urllib.request
from html import escape
from uuid import uuid4

from ._daemon import load_auth, rest_base

_EMAIL_ENDPOINT = "/rest/v1/plugin/email/send"
_GRAPHQL_PATH = "/graphql"

# Deep link into the Sledované dokumenty app (folders tab).
APP_LINK = "/sledovane-dokumenty?tab=folders"


# ── transport ────────────────────────────────────────────────────────────────


def _daemon_url():
    return os.environ.get("CODEXIS_PUBLIC_DAEMON_URL")


def _send_email(subject, body_text, body_html, recipients=None):
    daemon_url, auth = _daemon_url(), load_auth()
    if not daemon_url or not auth:
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
        rest_base(daemon_url) + _EMAIL_ENDPOINT,
        data=body,
        method="POST",
        headers={
            "Authorization": auth,
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


def _query_me_email():
    """Signed-in user's e-mail from the daemon (GraphQL `me`), or None. Best-effort."""
    daemon_url, auth = _daemon_url(), load_auth()
    if not daemon_url or not auth:
        return None
    body = json.dumps({"query": "query { me { email } }"}).encode("utf-8")
    try:
        req = urllib.request.Request(
            _graphql_url(),
            data=body,
            headers={"Content-Type": "application/json", "Authorization": auth},
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.load(resp)
        if not isinstance(data, dict):
            return None
        me = (data.get("data") or {}).get("me") or {}
        email = me.get("email")
        return email if isinstance(email, str) and email else None
    except Exception:
        return None


def resolve_to(recipients):
    """Compose the e-mail `to` list: the signed-in user is always included when
    there are extra recipients.

    - no extras -> None (daemon delivers to the authenticated user)
    - extras    -> [signed_in_user, *extras] with any extra matching the signed-in
      user (case-insensitive) dropped to avoid a duplicate, or just extras if `me`
      can't be resolved
    """
    extras = [r for r in (recipients or []) if str(r).strip()]
    if not extras:
        return None
    me = _query_me_email()
    if not me:
        return list(extras)
    me_key = me.strip().lower()
    return [me, *[e for e in extras if e.strip().lower() != me_key]]


def without_self(recipients):
    """Extra recipients with the signed-in user's own address removed.

    The signed-in user always receives the e-mail, so listing their own address
    as an extra is redundant (and shows as a duplicate pill in the UI). This is
    the store-time guard for the CLI and daemon write paths - the FE dialog
    already blocks it on entry. Case-insensitive. When the signed-in user can't
    be resolved, the list is returned unchanged (keep an extra rather than drop a
    real recipient on a transient lookup failure). No `me` lookup for an empty
    list.
    """
    items = [r for r in (recipients or []) if str(r).strip()]
    if not items:
        return items
    me = _query_me_email()
    if not me:
        return items
    me_key = me.strip().lower()
    return [r for r in items if str(r).strip().lower() != me_key]


def _send_inapp(message, *, action=None, link=None, ntype="legislation-changed"):
    daemon_url, auth = _daemon_url(), load_auth()
    if not daemon_url or not auth:
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
            "Authorization": auth,
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
        text_lines.append(f"Složka „{r['name']}“ - {len(r['changes'])} změn:")
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

    if settings is None:
        from . import settings as _settings
        settings = _settings.get_notify()
    subject, body_text, body_html = summary

    email_sent = inapp_sent = False
    if settings.get("email"):
        email_sent = _send_email(
            subject, body_text, body_html, recipients=resolve_to(settings.get("recipients"))
        )
    if settings.get("inApp"):
        inapp_sent = _send_inapp(
            subject, action="Zobrazit změny", link=APP_LINK
        )
    return {"email": email_sent, "inApp": inapp_sent, "changes": total}


# ── document-change notifications ───────────────────────────────────────────


def _email_shell(heading, inner_html):
    """Wrap inner HTML in the branded e-mail envelope shared by document notices.

    Mirrors the inline styling `build_summary` uses for folder-change e-mails.
    """
    return (
        '<div style="max-width:560px;font:14px sans-serif;color:#0a0a0a">'
        '<p style="margin:0;font:600 12px sans-serif;letter-spacing:.14em;'
        'text-transform:uppercase;color:#5ea500">Sledované dokumenty</p>'
        f'<h2 style="margin:4px 0 2px;font:400 22px Georgia,serif">{heading}</h2>'
        + inner_html
        + '<p style="margin-top:20px;font:13px sans-serif;color:#777">'
        "Otevřete aplikaci Sledované dokumenty v CODEXIS pro detail.</p></div>"
    )


def build_document_summary(changed):
    """Build (subject, body_text, body_html) for changed tracked documents, or None."""
    if not changed:
        return None
    n = len(changed)
    subject = f"Sledované dokumenty – {n} změn" if n != 1 else "Sledované dokumenty – 1 změna"
    names = [c.get("name") or c.get("codexisId") or "?" for c in changed]
    body_text = "Změny ve sledovaných dokumentech:\n" + "\n".join(f"- {x}" for x in names)
    inner = "".join(f"<li>{escape(x)}</li>" for x in names)
    body_html = _email_shell("Změny ve sledovaných dokumentech", f"<ul>{inner}</ul>")
    return subject, body_text, body_html


def notify_document_changes(changed, settings=None):
    """Send e-mail and/or in-app notifications for changed tracked documents.

    Returns {"email": bool, "inApp": bool, "changes": int}. Best-effort; never
    raises on transport failure.
    """
    summary = build_document_summary(changed)
    if summary is None:
        return {"email": False, "inApp": False, "changes": 0}
    if settings is None:
        from . import settings as _settings
        settings = _settings.get_notify()
    subject, body_text, body_html = summary
    email_sent = inapp_sent = False
    if settings.get("email"):
        email_sent = _send_email(
            subject, body_text, body_html, recipients=resolve_to(settings.get("recipients"))
        )
    if settings.get("inApp"):
        inapp_sent = _send_inapp(subject, action="Zobrazit změny", link=APP_LINK)
    return {"email": email_sent, "inApp": inapp_sent, "changes": len(changed)}
