"""CGI-layer tests for the notify-settings HTTP endpoint in
components/katastr/katastr.py: GET ?view=notify and POST save_notify_settings.

Loads the component binary in-process (mirrors _load_cli() for bin/katastr-cli)
so handle_get/handle_post can be called directly and module-level `notify` /
`notify_settings` references monkeypatched.
"""

import json

import pytest

from _helpers import _load_component_binary

_component = _load_component_binary()  # defs-only/stateless — load once, reuse across tests


@pytest.fixture
def home(tmp_path, monkeypatch):
    """Isolate notify-settings I/O to a tmp home, mirroring test_notify_settings.py."""
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    monkeypatch.setenv("HOME", str(tmp_path))
    ns = _component.notify_settings
    d = tmp_path / ".local" / "share" / ns.APP_ID / "settings"
    d.mkdir(parents=True)
    (d / "defaults.json").write_text(json.dumps(ns.FALLBACK_DEFAULTS), encoding="utf-8")
    return tmp_path


def _parse_emitted(capsys):
    """Split emit_json's raw CGI output into (status_or_None, payload dict)."""
    out = capsys.readouterr().out
    head, _, body = out.partition("\r\n\r\n")
    status = None
    for line in head.split("\r\n"):
        if line.startswith("Status:"):
            status = line[len("Status:"):].strip()
    return status, json.loads(body)


def test_get_view_notify_returns_settings_and_user_email(home, monkeypatch, capsys):
    monkeypatch.setattr(_component.notify, "_query_me_email", lambda: "me@firma.cz")
    _component.handle_get("view=notify")
    status, payload = _parse_emitted(capsys)
    assert status is None
    assert payload == {
        "mode": "notify",
        "settings": {"email": False, "inApp": True, "recipients": []},
        "userEmail": "me@firma.cz",
    }


def test_get_view_notify_user_email_blank_when_unavailable(home, monkeypatch, capsys):
    monkeypatch.setattr(_component.notify, "_query_me_email", lambda: None)
    _component.handle_get("view=notify")
    _, payload = _parse_emitted(capsys)
    assert payload["userEmail"] == ""


def test_get_view_notify_user_email_blank_on_lookup_error(home, monkeypatch, capsys):
    def _boom():
        raise RuntimeError("network down")
    monkeypatch.setattr(_component.notify, "_query_me_email", _boom)
    _component.handle_get("view=notify")
    _, payload = _parse_emitted(capsys)
    assert payload["userEmail"] == ""


def test_post_save_notify_settings_persists(home, capsys):
    _component.handle_post({
        "action": "save_notify_settings",
        "email": True,
        "inApp": False,
        "recipients": ["a@x.cz"],
    })
    status, payload = _parse_emitted(capsys)
    assert status is None
    assert payload == {
        "ok": True,
        "settings": {"email": True, "inApp": False, "recipients": ["a@x.cz"]},
    }
    on_disk = json.loads(
        (home / ".cdx" / "apps" / _component.notify_settings.APP_ID / "settings.json")
        .read_text()
    )
    assert on_disk["notify"] == {"email": True, "inApp": False, "recipients": ["a@x.cz"]}


def test_post_save_notify_settings_invalid_recipient_returns_400(home, capsys):
    _component.handle_post({
        "action": "save_notify_settings",
        "recipients": ["nope"],
    })
    status, payload = _parse_emitted(capsys)
    assert status == "400 Bad Request"
    assert payload["ok"] is False
    assert "error" in payload


def test_check_all_pushes_notifications_for_changes(home, monkeypatch, capsys):
    """FE check_all sends e-mail + in-app for detected changes, same as the CLI —
    so a change is notified regardless of which route first catches it."""
    results = [{"cislo_rizeni": "V-1/2026-101", "ok": True,
                "change": {"new_stav": "x"}, "error": None}]
    sent = {}
    monkeypatch.setattr(_component.tracking, "check_all", lambda: results)
    monkeypatch.setattr(_component.notify_settings, "get_notify",
                        lambda: {"email": True, "inApp": True, "recipients": ["a@x.cz"]})
    monkeypatch.setattr(_component.notify, "resolve_to", lambda r: r)
    monkeypatch.setattr(_component.notify, "email_change_summary",
                        lambda res, to=None: sent.update(email=(res, to)) or 1)
    monkeypatch.setattr(_component.notify, "inapp_change_summary",
                        lambda res: sent.update(inapp=res) or True)

    _component.handle_post({"action": "check_all"})
    status, payload = _parse_emitted(capsys)

    assert status is None
    assert payload == {"ok": True, "checked": 1, "new_changes": 1, "errors": []}
    assert sent["email"] == (results, ["a@x.cz"])
    assert sent["inapp"] == results


def test_check_all_send_failure_never_breaks_response(home, monkeypatch, capsys):
    """A failing notify send must not break the FE check response (best-effort)."""
    results = [{"cislo_rizeni": "V-1/2026-101", "ok": True,
                "change": {"new_stav": "x"}, "error": None}]

    def _boom(*a, **k):
        raise RuntimeError("smtp down")

    monkeypatch.setattr(_component.tracking, "check_all", lambda: results)
    monkeypatch.setattr(_component.notify_settings, "get_notify",
                        lambda: {"email": True, "inApp": True, "recipients": []})
    monkeypatch.setattr(_component.notify, "resolve_to", lambda r: r)
    monkeypatch.setattr(_component.notify, "email_change_summary", _boom)
    monkeypatch.setattr(_component.notify, "inapp_change_summary", _boom)

    _component.handle_post({"action": "check_all"})
    status, payload = _parse_emitted(capsys)

    assert status is None
    assert payload == {"ok": True, "checked": 1, "new_changes": 1, "errors": []}
    # No partial write on validation failure — set_notify() raises before
    # atomic_write_json(), so the user settings file was never created.
    settings_path = home / ".cdx" / "apps" / _component.notify_settings.APP_ID / "settings.json"
    assert not settings_path.exists()
