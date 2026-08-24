"""HTTP CGI notify wiring: GET ?view=notify shape + POST save_notify_settings.

Loads components/sledovana-judikatura/judikatura-cgi in-process (same harness
test_cli_notify.py uses for bin/cdx-sledovana-judikatura) so handle_get/handle_post
run directly against the `judikatura_app_dir` fixture's isolated state, and so
module-level `notify`/`notify_settings` references can be monkeypatched.
"""

import json
import os

import pytest
from sledovana_judikatura_core import notify_settings

from tests.harness.script_loader import load_script


def _load_component():
    path = os.path.join(
        os.path.dirname(__file__),
        "..",
        "..",
        "..",
        "components",
        "sledovana-judikatura",
        "judikatura-cgi",
    )
    return load_script("cdx_sledovana_judikatura_http", path)


_http = _load_component()  # defs-only/stateless - load once, reuse across tests


@pytest.fixture
def defaults(judikatura_app_dir, monkeypatch, tmp_path):
    """Pin notify_settings' defaults.json to a known, host-independent value."""
    shipped = tmp_path / "defaults.json"
    shipped.write_text(json.dumps(notify_settings.FALLBACK_DEFAULTS), encoding="utf-8")
    monkeypatch.setattr(notify_settings, "defaults_path", lambda: str(shipped))
    return shipped


def _parse(capsys):
    """Split a CGI emit_json() write into (status_or_None, decoded_body)."""
    raw = capsys.readouterr().out
    head, _, body = raw.partition("\r\n\r\n")
    status = None
    for line in head.split("\r\n"):
        if line.startswith("Status:"):
            status = line.split(":", 1)[1].strip()
    return status, json.loads(body)


# ── GET ?view=notify ─────────────────────────────────────────────────────────


def test_get_view_notify_shape_with_signed_in_user(defaults, monkeypatch, capsys):
    monkeypatch.setattr(_http.notify, "_query_me_email", lambda: "me@firma.cz")
    _http.handle_get("view=notify")
    status, body = _parse(capsys)
    assert status is None
    assert body == {
        "mode": "notify",
        "settings": {"email": False, "inApp": True, "recipients": []},
        "userEmail": "me@firma.cz",
    }


def test_get_view_notify_user_email_blank_when_unavailable(
    defaults, monkeypatch, capsys
):
    monkeypatch.setattr(_http.notify, "_query_me_email", lambda: None)
    _http.handle_get("view=notify")
    _, body = _parse(capsys)
    assert body["userEmail"] == ""


def test_get_view_notify_reflects_saved_settings(defaults, monkeypatch, capsys):
    notify_settings.set_notify(email=True, inApp=False, recipients=["a@x.cz"])
    monkeypatch.setattr(_http.notify, "_query_me_email", lambda: None)
    _http.handle_get("view=notify")
    _, body = _parse(capsys)
    assert body["settings"] == {"email": True, "inApp": False, "recipients": ["a@x.cz"]}


# ── POST save_notify_settings ────────────────────────────────────────────────


def test_post_save_notify_settings_persists(defaults, judikatura_app_dir, capsys):
    _http.handle_post(
        {
            "action": "save_notify_settings",
            "email": True,
            "inApp": False,
            "recipients": ["a@x.cz", "A@X.CZ"],
        }
    )
    status, body = _parse(capsys)
    assert status is None
    assert body == {
        "ok": True,
        "settings": {"email": True, "inApp": False, "recipients": ["a@x.cz"]},
    }
    on_disk = json.loads((judikatura_app_dir / "settings.json").read_text())
    assert on_disk["notify"] == {
        "email": True,
        "inApp": False,
        "recipients": ["a@x.cz"],
    }


def test_post_save_notify_settings_drops_signed_in_user(defaults, monkeypatch, capsys):
    # The UI already blocks adding your own address, but the CLI/automation can
    # store it. The handler must strip it so it isn't persisted (and legacy dupes
    # self-heal on the next save).
    monkeypatch.setattr(_http.notify, "_query_me_email", lambda: "me@firma.cz")
    _http.handle_post(
        {
            "action": "save_notify_settings",
            "email": True,
            "inApp": False,
            "recipients": ["ME@firma.cz", "a@x.cz"],
        }
    )
    _, body = _parse(capsys)
    assert body["settings"]["recipients"] == ["a@x.cz"]


def test_post_save_notify_settings_absent_recipients_left_untouched(
    defaults, monkeypatch, capsys
):
    # A save that omits `recipients` (e.g. toggling only the e-mail switch) must
    # not clear the stored list - self-stripping must not turn None into [].
    monkeypatch.setattr(_http.notify, "_query_me_email", lambda: "me@firma.cz")
    notify_settings.set_notify(email=True, inApp=True, recipients=["a@x.cz"])
    _http.handle_post({"action": "save_notify_settings", "email": False})
    _, body = _parse(capsys)
    assert body["settings"]["recipients"] == ["a@x.cz"]


def test_post_save_notify_settings_invalid_recipient_400(defaults, capsys):
    _http.handle_post(
        {
            "action": "save_notify_settings",
            "recipients": ["nope"],
        }
    )
    status, body = _parse(capsys)
    assert status == "400 Bad Request"
    assert body["ok"] is False
    assert "error" in body


def test_post_save_notify_settings_omitted_fields_leave_existing_untouched(
    defaults, capsys
):
    notify_settings.set_notify(email=True, inApp=True, recipients=["a@x.cz"])
    _http.handle_post({"action": "save_notify_settings", "inApp": False})
    _, body = _parse(capsys)
    assert body["settings"] == {"email": True, "inApp": False, "recipients": ["a@x.cz"]}
