"""E-mail + in-app summary of newly detected case law from a check run.

Sends only when new case law was actually found; a quiet run sends nothing.

E-mail uses the daemon's plugin e-mail API (POST /rest/v1/plugin/email/send).
Auth is the VM/daemon token the host writes to ~/.cdx/env/secret (the same token
cdxctl uses) — NOT CODEXIS_USER_API_TOKEN, which is a JWT for external Codexis
sources and is rejected by the gateway. The daemon URL comes from
CODEXIS_PUBLIC_DAEMON_URL. The recipient defaults to the authenticated user;
pass `to` to send elsewhere. Meant to run from a recurring automation.

A "result" is one topic's check outcome::

    {"topic_name": str, "report": <report dict>, "ok": bool, "error": str|None}

A result carries a change when it completed OK and its report found new case law
(``found_count`` > 0).
"""

import json
import os
import urllib.error
import urllib.request
from html import escape
from uuid import uuid4

from .exceptions import TopicError

_EMAIL_ENDPOINT = "/rest/v1/plugin/email/send"
# Auth for the daemon/gateway API lives in this file (written by the host), NOT in
# CODEXIS_USER_API_TOKEN — that is a JWT for external Codexis sources and the gateway
# rejects it (403). This is the same token cdxctl reads.
_SECRET_FILE = os.path.join(".cdx", "env", "secret")

# Deep link into the plugin's app + notification type tag for the bell.
APP_LINK = "/sledovana-judikatura"
_NOTIFICATION_TYPE = "case-law-changed"


def _load_auth() -> str | None:
    """Bearer authorization header from ~/.cdx/env/secret, or None if unavailable."""
    path = os.path.join(os.path.expanduser("~"), _SECRET_FILE)
    try:
        with open(path, encoding="utf-8") as f:
            token = f.read().strip()
    except OSError:
        return None
    if not token:
        return None
    if token.lower().startswith("authorization:"):
        token = token[len("Authorization:"):].strip()
    if token[:7].lower() == "bearer ":
        return token
    return f"Bearer {token}"


def _query_me_email() -> str | None:
    """Signed-in user's e-mail from the daemon (GraphQL `me`), or None. Best-effort."""
    daemon_url = os.environ.get("CODEXIS_PUBLIC_DAEMON_URL")
    auth = _load_auth()
    if not daemon_url or not auth:
        return None
    body = json.dumps({"query": "query { me { email } }"}).encode("utf-8")
    try:
        req = urllib.request.Request(
            daemon_url,
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


def resolve_to(recipients: list | None) -> list | None:
    """Compose the e-mail `to` list (variant 2): the signed-in user is always
    included when there are extra recipients.

    - no extras  -> None (daemon delivers to the authenticated user)
    - extras     -> [signed_in_user, *extras] with any extra matching the signed-in
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


def _rest_base(daemon_url: str) -> str:
    """REST base from CODEXIS_PUBLIC_DAEMON_URL, which is the GraphQL URL (…/graphql)."""
    base = daemon_url.rstrip("/")
    if base.endswith("/graphql"):
        base = base[: -len("/graphql")].rstrip("/")
    return base


def _send_email(subject: str, body_text: str, body_html: str | None = None, to: list | None = None) -> None:
    daemon_url = os.environ.get("CODEXIS_PUBLIC_DAEMON_URL")
    auth = _load_auth()
    if not daemon_url or not auth:
        raise TopicError(
            "E-mail nelze odeslat — chybí CODEXIS_PUBLIC_DAEMON_URL nebo přihlašovací "
            "token (~/.cdx/env/secret). Spouštějte přes automatizaci v CODEXIS."
        )

    payload: dict = {"subject": subject, "bodyText": body_text}
    if body_html:
        payload["bodyHtml"] = body_html
    # Omit `to` → the daemon defaults the recipient to the authenticated user.
    if to:
        payload["to"] = list(to)

    boundary = "----judikatura-" + uuid4().hex
    body = (
        f"--{boundary}\r\n"
        'Content-Disposition: form-data; name="payload"\r\n'
        "Content-Type: application/json; charset=utf-8\r\n\r\n"
        f"{json.dumps(payload, ensure_ascii=False)}\r\n"
        f"--{boundary}--\r\n"
    ).encode("utf-8")

    req = urllib.request.Request(
        _rest_base(daemon_url) + _EMAIL_ENDPOINT,
        data=body,
        method="POST",
        headers={
            "Authorization": auth,
            "Content-Type": f"multipart/form-data; boundary={boundary}",
        },
    )
    try:
        urllib.request.urlopen(req, timeout=30).close()
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", "replace")[:300]
        raise TopicError(f"E-mail se nepodařilo odeslat (HTTP {exc.code}): {detail}")
    except urllib.error.URLError as exc:
        raise TopicError(f"E-mail se nepodařilo odeslat: {exc.reason}")
    except OSError as exc:
        # urlopen only wraps connect/send OSErrors in URLError; a read-phase
        # timeout / reset / SSL error is raised bare. Catch it so the CLI's
        # TopicError handler reports it instead of crashing with a traceback.
        raise TopicError(f"E-mail se nepodařilo odeslat: {exc}")


def _changed(results: list) -> list:
    """Results that completed OK and whose report found new case law."""
    return [
        r for r in results
        if r.get("ok") and (r.get("report") or {}).get("found_count", 0) > 0
    ]


_FOOTER = "Otevřete aplikaci Sledovaná judikatura v CODEXIS pro detail."


def _email_shell(heading: str, inner_html: str) -> str:
    """Wrap inner HTML in the shared branded e-mail envelope."""
    return (
        '<div style="max-width:560px;font:14px sans-serif;color:#0a0a0a">'
        '<p style="margin:0;font:600 12px sans-serif;letter-spacing:.14em;'
        'text-transform:uppercase;color:#5ea500">Sledovaná judikatura</p>'
        f'<h2 style="margin:4px 0 2px;font:400 22px Georgia,serif">{heading}</h2>'
        + inner_html
        + f'<p style="margin-top:20px;font:13px sans-serif;color:#777">{_FOOTER}</p></div>'
    )


def _doc_title(doc: dict) -> str:
    return doc.get("title") or doc.get("spZn") or doc.get("codexisId") or "?"


def _report_lines(report: dict) -> list:
    """Plain-text lines describing what a single report found."""
    found = report.get("found_count", 0)
    period_from = report.get("period_from") or "?"
    period_to = report.get("period_to") or "?"
    lines = [f"Nalezeno {found} nových rozhodnutí (období {period_from} → {period_to})."]
    for area in report.get("areas", []):
        docs = area.get("documents", [])
        if not docs:
            continue
        lines.append(f"{area.get('name', '')}:")
        for d in docs:
            suffix = " (mění dosavadní výklad)" if d.get("changes_established_view") else ""
            lines.append(f"  • {_doc_title(d)}{suffix}")
    overall = report.get("overall_summary")
    if overall:
        lines.append(f"Souhrn: {overall}")
    return lines


def _report_html(report: dict) -> str:
    """HTML fragment (lead paragraph + decision list) for a single report."""
    found = report.get("found_count", 0)
    period_from = report.get("period_from") or "?"
    period_to = report.get("period_to") or "?"
    lead = f"Nalezeno {found} nových rozhodnutí (období {period_from} → {period_to})."
    items = []
    for area in report.get("areas", []):
        for d in area.get("documents", []):
            suffix = (
                ' <span style="color:#777">(mění dosavadní výklad)</span>'
                if d.get("changes_established_view") else ""
            )
            items.append(f'<li style="margin:2px 0">{escape(_doc_title(d))}{suffix}</li>')
    ul = (
        f'<ul style="margin:0;padding-left:18px;font:14px/1.5 sans-serif;color:#333">'
        f'{"".join(items)}</ul>'
        if items else ""
    )
    return (
        f'<p style="margin:0 0 6px;font:14px sans-serif;color:#555">{escape(lead)}</p>{ul}'
    )


def _change_summary(changed: list):
    """(subject, body_text, body_html) for topics that found new case law."""
    if len(changed) == 1:
        subject = f"Sledovaná judikatura: nová judikatura u tématu {changed[0]['topic_name']}"
    else:
        subject = f"Sledovaná judikatura: nová judikatura u {len(changed)} témat"

    text_lines = ["Sledování judikatury zaznamenalo novou judikaturu:", ""]
    html_blocks = []
    for r in changed:
        text_lines.append(f"{r['topic_name']}:")
        text_lines.extend(f"  {line}" for line in _report_lines(r["report"]))
        text_lines.append("")
        html_blocks.append(
            f'<h3 style="margin:18px 0 6px;font:600 15px sans-serif;color:#0a0a0a">'
            f"{escape(r['topic_name'])}</h3>"
            + _report_html(r["report"])
        )
    text_lines.append(_FOOTER)

    return subject, "\n".join(text_lines), _email_shell(
        "Nová judikatura u sledovaných témat", "".join(html_blocks)
    )


def build_summary(results: list):
    """Build (subject, body_text, body_html) for topics with new case law, or None."""
    changed = _changed(results)
    if not changed:
        return None
    return _change_summary(changed)


def email_change_summary(results: list, to: list | None = None):
    """Send one summary e-mail for topics that found new case law.

    Returns the number of topics with new case law (0 → nothing sent). Pass
    `to` to override the recipient (default: the signed-in user).
    """
    changed = _changed(results)
    if not changed:
        return 0
    _send_email(*_change_summary(changed), to=to)
    return len(changed)


def _send_inapp(message: str, *, action: str | None = None, link: str | None = None) -> bool:
    """Create an in-app (notification-center bell) notification via the daemon's
    GraphQL createNotification. Best-effort: returns False on missing config or a
    transport error, never raises (a notification must not fail the check)."""
    graphql_url = os.environ.get("CODEXIS_PUBLIC_DAEMON_URL")
    auth = _load_auth()
    if not graphql_url or not auth:
        return False
    mutation = (
        "mutation Create($input: CreateNotificationInput!) "
        "{ createNotification(input: $input) { id } }"
    )
    notification_input = {"message": message, "type": _NOTIFICATION_TYPE}
    if action:
        notification_input["action"] = action
    if link:
        notification_input["link"] = link
    body = json.dumps(
        {"query": mutation, "variables": {"input": notification_input}},
        ensure_ascii=False,
    ).encode("utf-8")
    req = urllib.request.Request(
        graphql_url,
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
    except (urllib.error.URLError, OSError, ValueError):
        return False


def inapp_change_summary(results: list) -> bool:
    """Send an in-app (bell) notification only when new case law was found
    (no heartbeat bell). Returns True if sent."""
    changed = _changed(results)
    if not changed:
        return False
    subject = _change_summary(changed)[0]
    return _send_inapp(subject, link=APP_LINK)
