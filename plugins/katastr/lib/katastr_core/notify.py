"""E-mail a summary of a proceeding-tracking check run.

Every run e-mails something — the newly detected changes, or a "no new changes"
heartbeat so the user sees the watchdog is alive.

Uses the daemon's plugin e-mail API (POST /rest/v1/plugin/email/send). Auth is the
VM/daemon token the host writes to ~/.cdx/env/secret (the same token cdxctl uses) —
NOT CODEXIS_USER_API_TOKEN, which is a JWT for external Codexis sources and is
rejected by the gateway. The daemon URL comes from CODEXIS_PUBLIC_DAEMON_URL. The
recipient defaults to the authenticated user; pass `to` to send elsewhere. Meant to
run from a recurring automation.
"""

import json
import os
import urllib.error
import urllib.request
from html import escape
from uuid import uuid4

from .exceptions import KatastrError
from .tracking import describe_change

_EMAIL_ENDPOINT = "/rest/v1/plugin/email/send"
# Auth for the daemon/gateway API lives in this file (written by the host), NOT in
# CODEXIS_USER_API_TOKEN — that is a JWT for external Codexis sources and the gateway
# rejects it (403). This is the same token cdxctl reads.
_SECRET_FILE = os.path.join(".cdx", "env", "secret")


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
    if token.startswith("Authorization:"):
        token = token[len("Authorization:"):].strip()
    if token[:7].lower() == "bearer ":
        return token
    return f"Bearer {token}"


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
        raise KatastrError(
            "E-mail nelze odeslat — chybí CODEXIS_PUBLIC_DAEMON_URL nebo přihlašovací "
            "token (~/.cdx/env/secret). Spouštějte přes automatizaci v CODEXIS."
        )

    payload: dict = {"subject": subject, "bodyText": body_text}
    if body_html:
        payload["bodyHtml"] = body_html
    # Omit `to` → the daemon defaults the recipient to the authenticated user.
    if to:
        payload["to"] = list(to)

    boundary = "----katastr-" + uuid4().hex
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
        raise KatastrError(f"E-mail se nepodařilo odeslat (HTTP {exc.code}): {detail}")
    except urllib.error.URLError as exc:
        raise KatastrError(f"E-mail se nepodařilo odeslat: {exc.reason}")
    except OSError as exc:
        # urlopen only wraps connect/send OSErrors in URLError; a read-phase
        # timeout / reset / SSL error is raised bare. Catch it so the CLI's
        # KatastrError handler reports it instead of crashing with a traceback.
        raise KatastrError(f"E-mail se nepodařilo odeslat: {exc}")


def _changed(results: list) -> list:
    """Results that completed OK and carry a newly detected change."""
    return [r for r in results if r.get("ok") and r.get("change")]


_FOOTER = "Otevřete aplikaci Katastr v CODEXIS pro detail."


def _email_shell(heading: str, inner_html: str) -> str:
    """Wrap inner HTML in the shared branded e-mail envelope."""
    return (
        '<div style="max-width:560px;font:14px sans-serif;color:#0a0a0a">'
        '<p style="margin:0;font:600 12px sans-serif;letter-spacing:.14em;'
        'text-transform:uppercase;color:#5ea500">Hlídač katastrálních řízení</p>'
        f'<h2 style="margin:4px 0 2px;font:400 22px Georgia,serif">{heading}</h2>'
        + inner_html
        + f'<p style="margin-top:20px;font:13px sans-serif;color:#777">{_FOOTER}</p></div>'
    )


def _no_change_summary(results: list):
    """Heartbeat (subject, body_text, body_html) for a run with no new changes.

    Sent so the user sees the watchdog is alive even on a quiet run. Notes any
    proceedings whose check failed rather than silently claiming "all clear".
    """
    ok = [r for r in results if r.get("ok")]
    errored = [r for r in results if not r.get("ok")]

    subject = "Katastr: žádné nové změny" if ok else "Katastr: kontrolu se nepodařilo dokončit"

    text_lines = ["Hlídač katastrálních řízení — pravidelná kontrola:", ""]
    inner = ""
    if ok:
        text_lines.append(f"Zkontrolováno {len(ok)} řízení — žádné nové změny.")
        inner += (
            '<p style="margin:0;font:14px/1.5 sans-serif;color:#333">'
            f"Zkontrolováno {len(ok)} řízení — žádné nové změny.</p>"
        )
    if errored:
        text_lines.append(f"U {len(errored)} řízení se kontrola nezdařila.")
        inner += (
            '<p style="margin:6px 0 0;font:14px/1.5 sans-serif;color:#b45309">'
            f"U {len(errored)} řízení se kontrola nezdařila.</p>"
        )
    text_lines += ["", _FOOTER]

    return subject, "\n".join(text_lines), _email_shell("Žádné nové změny", inner)


def _change_summary(changed: list):
    """(subject, body_text, body_html) for proceedings with a newly detected change."""
    if len(changed) == 1:
        subject = f"Katastr: změna u řízení {changed[0]['cislo_rizeni']}"
    else:
        subject = f"Katastr: změny u {len(changed)} sledovaných řízení"

    text_lines = ["Hlídač katastrálních řízení zaznamenal změny:", ""]
    html_blocks = []
    for r in changed:
        detail = describe_change(r["change"])
        text_lines.append(f"{r['cislo_rizeni']}:")
        text_lines.extend(f"  {line}" for line in detail)
        text_lines.append("")
        items = "".join(f'<li style="margin:2px 0">{escape(line)}</li>' for line in detail)
        html_blocks.append(
            f'<h3 style="margin:18px 0 6px;font:600 15px sans-serif;color:#0a0a0a">'
            f"{escape(r['cislo_rizeni'])}</h3>"
            f'<ul style="margin:0;padding-left:18px;font:14px/1.5 sans-serif;color:#333">{items}</ul>'
        )
    text_lines.append(_FOOTER)

    return subject, "\n".join(text_lines), _email_shell(
        "Změny u sledovaných řízení", "".join(html_blocks)
    )


def build_summary(results: list):
    """Build (subject, body_text, body_html) for a check run, or None if nothing was checked.

    Always summarises a run: the detected changes, or a "no new changes"
    heartbeat. Returns None only when there were no proceedings to check.
    """
    if not results:
        return None
    changed = _changed(results)
    return _change_summary(changed) if changed else _no_change_summary(results)


def email_change_summary(results: list, to: list | None = None):
    """Send one summary e-mail for a check run.

    Sends the detected changes, or a "no new changes" heartbeat so the user sees
    the watchdog is alive. Pass `to` to override the recipient (default: the
    signed-in user). Returns the number of proceedings with a new change (0 for a
    heartbeat), or None when there was nothing to check (no e-mail sent).
    """
    if not results:
        return None
    changed = _changed(results)
    summary = _change_summary(changed) if changed else _no_change_summary(results)
    _send_email(*summary, to=to)
    return len(changed)
