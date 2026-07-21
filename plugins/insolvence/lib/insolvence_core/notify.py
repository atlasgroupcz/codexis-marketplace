"""E-mail the user a summary of newly detected insolvency changes.

Uses the daemon's plugin e-mail API (POST /rest/v1/plugin/email/send). Auth is the
VM/daemon token the host writes to ~/.cdx/env/secret (the same token cdxctl uses) -
NOT CODEXIS_USER_API_TOKEN, which is a JWT for external Codexis sources and is
rejected by the gateway. The daemon URL comes from CODEXIS_PUBLIC_DAEMON_URL, whose
value is the GraphQL URL. The recipient defaults to the authenticated user; pass
`to` to send elsewhere. Meant to run from a recurring CODEXIS automation.
"""

import json
import os
import urllib.error
import urllib.request
from html import escape
from uuid import uuid4

from .exceptions import InsolvenceError

_EMAIL_ENDPOINT = "/rest/v1/plugin/email/send"
_SECRET_FILE = os.path.join(".cdx", "env", "secret")

# Deep link into the plugin's app + notification type tag for the bell.
APP_LINK = "/insolvence"
_NOTIFICATION_TYPE = "insolvency-changed"


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


def without_self(recipients: list | None) -> list:
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
        raise InsolvenceError(
            "E-mail nelze odeslat - chybí CODEXIS_PUBLIC_DAEMON_URL nebo daemon "
            "token (~/.cdx/env/secret). Spouštějte přes automatizaci v CODEXIS."
        )

    payload: dict = {"subject": subject, "bodyText": body_text}
    if body_html:
        payload["bodyHtml"] = body_html
    # Omit `to` → the daemon defaults the recipient to the authenticated user.
    if to:
        payload["to"] = list(to)

    boundary = "----insolvence-" + uuid4().hex
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
        raise InsolvenceError(f"E-mail se nepodařilo odeslat (HTTP {exc.code}): {detail}")
    except urllib.error.URLError as exc:
        raise InsolvenceError(f"E-mail se nepodařilo odeslat: {exc.reason}")
    except OSError as exc:
        # urlopen only wraps connect/send OSErrors in URLError; a read-phase
        # timeout / reset / SSL error is raised bare. Catch it so the CLI's
        # InsolvenceError handler reports it instead of crashing with a traceback.
        raise InsolvenceError(f"E-mail se nepodařilo odeslat: {exc}")


def build_summary(results: list) -> tuple[str, str, str] | None:
    """Build (subject, body_text, body_html) for subjects with new changes, or None."""
    changed = [r for r in results if r.get("ok") and r.get("changes")]
    total = sum(len(r["changes"]) for r in changed)
    if total == 0:
        return None

    if len(changed) == 1 and total == 1:
        subject = f"Insolvence: nová změna - {changed[0]['display_name']}"
    else:
        subject = f"Insolvence: {total} nových změn u {len(changed)} sledovaných subjektů"

    text_lines = ["Hlídač insolvencí zaznamenal nové změny:", ""]
    html_rows = []
    for r in changed:
        text_lines.append(f"{r['display_name']} - {len(r['changes'])} nových změn:")
        for c in r["changes"]:
            text_lines.append(f"  • {c.get('popis') or c.get('typ') or ''}")
        text_lines.append("")
        items = "".join(
            f'<li style="margin:2px 0">{escape(c.get("popis") or c.get("typ") or "")}</li>'
            for c in r["changes"]
        )
        html_rows.append(
            f'<h3 style="margin:18px 0 6px;font:600 15px sans-serif;color:#0a0a0a">'
            f"{escape(r['display_name'])}</h3>"
            f'<ul style="margin:0;padding-left:18px;font:14px/1.5 sans-serif;color:#333">{items}</ul>'
        )
    text_lines.append("Otevřete aplikaci Hlídač insolvencí v CODEXIS pro detail.")

    body_html = (
        '<div style="max-width:560px;font:14px sans-serif;color:#0a0a0a">'
        '<p style="margin:0;font:600 12px sans-serif;letter-spacing:.14em;'
        'text-transform:uppercase;color:#5ea500">Hlídač insolvencí</p>'
        '<h2 style="margin:4px 0 2px;font:400 22px Georgia,serif">'
        "Nové změny v insolvenčním rejstříku</h2>"
        + "".join(html_rows)
        + '<p style="margin-top:20px;font:13px sans-serif;color:#777">'
        "Otevřete aplikaci Hlídač insolvencí v CODEXIS pro detail.</p></div>"
    )
    return subject, "\n".join(text_lines), body_html


def email_change_summary(results: list, to: list | None = None) -> int:
    """Send one summary e-mail for subjects with newly detected changes.

    Returns the number of changes reported (0 → nothing to report, no e-mail sent).
    Pass `to` to override the recipient (default: the signed-in user).
    """
    summary = build_summary(results)
    if summary is None:
        return 0
    subject, body_text, body_html = summary
    _send_email(subject, body_text, body_html, to=to)
    return sum(len(r["changes"]) for r in results if r.get("ok") and r.get("changes"))


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
    """Send one in-app notification summarizing new changes. Returns True if sent
    (there were changes), False otherwise. Best-effort."""
    summary = build_summary(results)
    if summary is None:
        return False
    subject = summary[0]
    return _send_inapp(subject, link=APP_LINK)
