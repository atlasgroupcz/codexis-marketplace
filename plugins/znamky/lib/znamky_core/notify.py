"""E-mail the user a summary of newly detected similar trademarks.

Uses the daemon's plugin e-mail API (POST /rest/v1/plugin/email/send). The
daemon injects CODEXIS_PUBLIC_DAEMON_URL and CODEXIS_USER_API_TOKEN into every
plugin shell and defaults the recipient to the authenticated user, so no address
or secret handling is needed here. Meant to run from a recurring automation.
"""

import json
import os
import urllib.error
import urllib.request
from html import escape
from uuid import uuid4

from .exceptions import ZnamkyError

_EMAIL_ENDPOINT = "/rest/v1/plugin/email/send"
_TIER_LABEL = {"high": "vysoké riziko", "medium": "střední riziko", "low": "nízké riziko"}


def _send_email(subject: str, body_text: str, body_html: str = None) -> None:
    daemon_url = os.environ.get("CODEXIS_PUBLIC_DAEMON_URL")
    token = os.environ.get("CODEXIS_USER_API_TOKEN")
    if not daemon_url or not token:
        raise ZnamkyError(
            "E-mail nelze odeslat — chybí CODEXIS_PUBLIC_DAEMON_URL nebo "
            "CODEXIS_USER_API_TOKEN. Spouštějte přes automatizaci v CODEXIS."
        )

    payload = {"subject": subject, "bodyText": body_text}
    if body_html:
        payload["bodyHtml"] = body_html

    boundary = "----znamky-" + uuid4().hex
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
        raise ZnamkyError(f"E-mail se nepodařilo odeslat (HTTP {exc.code}): {detail}")
    except urllib.error.URLError as exc:
        raise ZnamkyError(f"E-mail se nepodařilo odeslat: {exc.reason}")


def _collision_line(c: dict) -> str:
    name = c.get("mark_text") or "(obrazová známka)"
    tier = _TIER_LABEL.get(c.get("tier", ""), "")
    applicant = c.get("applicant", "")
    bits = [name]
    if applicant:
        bits.append(f"přihlašovatel {applicant}")
    if tier:
        bits.append(tier)
    return " — ".join(bits)


def build_summary(results: list):
    """Build (subject, body_text, body_html) for marks with new collisions, or None."""
    changed = [r for r in results if r.get("ok") and r.get("new_collisions")]
    total = sum(len(r["new_collisions"]) for r in changed)
    if total == 0:
        return None

    if len(changed) == 1 and total == 1:
        subject = f"Ochrana známek: nová podobná známka — {changed[0]['display_name']}"
    else:
        subject = f"Ochrana známek: {total} nových podobných známek u {len(changed)} sledovaných"

    text_lines = ["Hlídač ochranných známek našel nové podobné přihlášky/zápisy:", ""]
    html_rows = []
    for r in changed:
        text_lines.append(f"{r['display_name']} — {len(r['new_collisions'])} nových:")
        for c in r["new_collisions"]:
            text_lines.append(f"  • {_collision_line(c)}")
        text_lines.append("")
        items = "".join(
            f'<li style="margin:2px 0">{escape(_collision_line(c))}</li>'
            for c in r["new_collisions"]
        )
        html_rows.append(
            f'<h3 style="margin:18px 0 6px;font:600 15px sans-serif;color:#0a0a0a">'
            f"{escape(r['display_name'])}</h3>"
            f'<ul style="margin:0;padding-left:18px;font:14px/1.5 sans-serif;color:#333">{items}</ul>'
        )
    text_lines.append("Otevřete aplikaci Hlídač ochranných známek v CODEXIS pro detail a posouzení.")

    body_html = (
        '<div style="max-width:560px;font:14px sans-serif;color:#0a0a0a">'
        '<p style="margin:0;font:600 12px sans-serif;letter-spacing:.14em;'
        'text-transform:uppercase;color:#5ea500">Hlídač ochranných známek</p>'
        '<h2 style="margin:4px 0 2px;font:400 22px Georgia,serif">'
        "Nové podobné ochranné známky</h2>"
        + "".join(html_rows)
        + '<p style="margin-top:20px;font:13px sans-serif;color:#777">'
        "Otevřete aplikaci Hlídač ochranných známek v CODEXIS pro detail a posouzení.</p></div>"
    )
    return subject, "\n".join(text_lines), body_html


def email_change_summary(results: list) -> int:
    """Send one summary e-mail for marks with newly detected collisions.

    Returns the number of new collisions reported (0 → nothing sent).
    """
    summary = build_summary(results)
    if summary is None:
        return 0
    subject, body_text, body_html = summary
    _send_email(subject, body_text, body_html)
    return sum(len(r["new_collisions"]) for r in results if r.get("ok") and r.get("new_collisions"))
