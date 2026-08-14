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
# Identifies this plugin's app so the bell's deep link can be resolved to the
# daemon's /apps/<applicationId> route; see _resolve_app_link below.
_PLUGIN_NAME = "codexis"
_COMPONENT_ID = "sledovane-dokumenty"
_LINK_QUERY = "?tab=folders"


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
    ).encode()
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


def _graphql(query):
    """POST a query to the daemon's GraphQL endpoint and return its `data` object.

    Best-effort: missing config or any transport/parse error yields None, so
    callers can treat "no daemon" and "bad answer" alike."""
    daemon_url, auth = _daemon_url(), load_auth()
    if not daemon_url or not auth:
        return None
    try:
        req = urllib.request.Request(
            _graphql_url(),
            data=json.dumps({"query": query}).encode("utf-8"),
            headers={"Content-Type": "application/json", "Authorization": auth},
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.load(resp)
    except Exception:
        return None
    return data.get("data") if isinstance(data, dict) else None


def _query_me_email():
    """Signed-in user's e-mail from the daemon (GraphQL `me`), or None. Best-effort."""
    me = (_graphql("query { me { email } }") or {}).get("me") or {}
    email = me.get("email")
    return email if isinstance(email, str) and email else None


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


# ── app deep link ─────────────────────────────────────────────────────────────
# The bell navigates relative links through the frontend router, whose only route
# for a plugin app is /apps/$applicationId - an opaque daemon node id, not a slug.
# A hand-written /<slug> matches no route and lands on a 404. The id embeds the
# marketplace install uuid, which the plugin has no way to know (it is absent from
# the environment and from ~/.cdx state), so it is looked up at send time from the
# daemon's `applications` query - scoped to the signed-in user, the same principal
# this plugin's daemon token already authenticates as.
#
# The id is never parsed, only carried: the query also returns contentUrl, which
# the daemon builds as /rest/plugin-components/<pluginNodeId>/<appId>/, so the
# component id is readable as its last path segment. Matching on that avoids
# decoding the daemon's internal node-id encoding, which this repo has no
# contract with.


def _component_id_of(content_url):
    """The component id a contentUrl points at - its last path segment.

    The daemon percent-encodes that segment, but component ids are kebab-case
    ASCII (repo convention), for which the encoding is the identity, so the
    segment is compared as-is."""
    trimmed = (content_url or "").rstrip("/")
    return trimmed.rsplit("/", 1)[-1] if trimmed else None


def _pick_application(applications, plugin_name, component_id):
    """The application id owned by `plugin_name`'s `component_id`.

    Both halves matter: one plugin may publish several components (codexis ships
    sledovana-judikatura and sledovane-dokumenty), and two marketplaces can carry
    same-named components."""
    if not isinstance(applications, list):
        return None
    for app in applications:
        if not isinstance(app, dict):
            continue
        app_id = app.get("id")
        plugin = app.get("plugin") or {}
        content_url = app.get("contentUrl")
        if not isinstance(app_id, str) or plugin.get("name") != plugin_name:
            continue
        if (
            isinstance(content_url, str)
            and _component_id_of(content_url) == component_id
        ):
            return app_id
    return None


def _resolve_app_link():
    """/apps/<applicationId> for this plugin's component, or None. Best-effort: an
    unresolvable link is omitted rather than sent broken - an unclickable bell item
    beats one that navigates into a 404."""
    data = _graphql("query { applications { id contentUrl plugin { name } } }") or {}
    found = _pick_application(data.get("applications"), _PLUGIN_NAME, _COMPONENT_ID)
    return f"/apps/{found}{_LINK_QUERY}" if found else None


def _send_inapp(message, *, link=None, ntype="legislation-changed"):
    daemon_url, auth = _daemon_url(), load_auth()
    if not daemon_url or not auth:
        return False

    mutation = (
        "mutation Create($input: CreateNotificationInput!) "
        "{ createNotification(input: $input) { id } }"
    )
    notification_input = {"message": message, "type": ntype}
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
            + (
                f' <span style="color:#777">(účinné od {escape(c["effective_on"])})</span>'
                if c.get("effective_on")
                else ""
            )
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
            subject,
            body_text,
            body_html,
            recipients=resolve_to(settings.get("recipients")),
        )
    if settings.get("inApp"):
        inapp_sent = _send_inapp(subject, link=_resolve_app_link())
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
    subject = (
        f"Sledované dokumenty - {n} změn" if n != 1 else "Sledované dokumenty - 1 změna"
    )
    names = [c.get("name") or c.get("codexisId") or "?" for c in changed]
    body_text = "Změny ve sledovaných dokumentech:\n" + "\n".join(
        f"- {x}" for x in names
    )
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
            subject,
            body_text,
            body_html,
            recipients=resolve_to(settings.get("recipients")),
        )
    if settings.get("inApp"):
        inapp_sent = _send_inapp(subject, link=_resolve_app_link())
    return {"email": email_sent, "inApp": inapp_sent, "changes": len(changed)}
