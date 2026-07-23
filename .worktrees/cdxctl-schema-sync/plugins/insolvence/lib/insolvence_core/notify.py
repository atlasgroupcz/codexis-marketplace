"""E-mail the user a summary of newly detected insolvency changes.

Uses the daemon's plugin e-mail API (POST /rest/v1/plugin/email/send). The daemon
injects CODEXIS_PUBLIC_DAEMON_URL and CODEXIS_USER_API_TOKEN into every plugin
shell, and the recipient defaults to the authenticated user — so no address or
secret handling is needed here. Meant to run from a recurring CODEXIS automation.
"""

import json
import os
import urllib.error
import urllib.request
from html import escape
from uuid import uuid4

from .exceptions import InsolvenceError

_EMAIL_ENDPOINT = "/rest/v1/plugin/email/send"


def _send_email(subject: str, body_text: str, body_html: str | None = None) -> None:
    daemon_url = os.environ.get("CODEXIS_PUBLIC_DAEMON_URL")
    token = os.environ.get("CODEXIS_USER_API_TOKEN")
    if not daemon_url or not token:
        raise InsolvenceError(
            "E-mail nelze odeslat — chybí CODEXIS_PUBLIC_DAEMON_URL nebo "
            "CODEXIS_USER_API_TOKEN. Spouštějte přes automatizaci v CODEXIS."
        )

    payload: dict = {"subject": subject, "bodyText": body_text}
    if body_html:
        payload["bodyHtml"] = body_html

    boundary = "----insolvence-" + uuid4().hex
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
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", "replace")[:300]
        raise InsolvenceError(f"E-mail se nepodařilo odeslat (HTTP {exc.code}): {detail}")
    except urllib.error.URLError as exc:
        raise InsolvenceError(f"E-mail se nepodařilo odeslat: {exc.reason}")


def build_summary(results: list) -> tuple[str, str, str] | None:
    """Build (subject, body_text, body_html) for subjects with new changes, or None."""
    changed = [r for r in results if r.get("ok") and r.get("changes")]
    total = sum(len(r["changes"]) for r in changed)
    if total == 0:
        return None

    if len(changed) == 1 and total == 1:
        subject = f"Insolvence: nová změna — {changed[0]['display_name']}"
    else:
        subject = f"Insolvence: {total} nových změn u {len(changed)} sledovaných subjektů"

    text_lines = ["Hlídač insolvencí zaznamenal nové změny:", ""]
    html_rows = []
    for r in changed:
        text_lines.append(f"{r['display_name']} — {len(r['changes'])} nových změn:")
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


def email_change_summary(results: list) -> int:
    """Send one summary e-mail for subjects with newly detected changes.

    Returns the number of changes reported (0 → nothing to report, no e-mail sent).
    """
    summary = build_summary(results)
    if summary is None:
        return 0
    subject, body_text, body_html = summary
    _send_email(subject, body_text, body_html)
    return sum(len(r["changes"]) for r in results if r.get("ok") and r.get("changes"))
