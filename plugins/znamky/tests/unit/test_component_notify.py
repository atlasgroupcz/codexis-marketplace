"""HTTP-endpoint tests for the znamky CGI component's notify settings surface:

- GET ?view=notify -> {"mode": "notify", "settings": ..., "userEmail": ...}
- POST action=save_notify_settings -> persists via notify_settings.set_notify

Loads components/znamky/znamky.py directly (it isn't an importable package -
it's the CGI entrypoint) and calls handle_get/handle_post, capturing the
emit_json output written to stdout. Isolates settings I/O to a tmp HOME, same
as test_notify_settings.py's `home` fixture.
"""

import importlib.util
import json
from pathlib import Path

import pytest

from znamky_core import notify_settings as ns

_COMPONENT_PATH = Path(__file__).resolve().parents[2] / "components" / "znamky" / "znamky.py"


def _load_component():
    spec = importlib.util.spec_from_file_location("znamky_component", _COMPONENT_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _payload(capsys):
    out = capsys.readouterr().out
    return json.loads(out.split("\r\n\r\n", 1)[1])


@pytest.fixture
def component(tmp_path, monkeypatch):
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    monkeypatch.setenv("HOME", str(tmp_path))
    d = tmp_path / ".local" / "share" / ns.APP_ID / "settings"
    d.mkdir(parents=True)
    (d / "defaults.json").write_text(json.dumps(ns.FALLBACK_DEFAULTS), encoding="utf-8")
    return _load_component()


def test_get_view_notify_returns_settings_and_email(component, capsys, monkeypatch):
    monkeypatch.setattr(component.notify, "_query_me_email", lambda: "me@firma.cz")
    component.handle_get("view=notify")
    payload = _payload(capsys)
    assert payload["mode"] == "notify"
    assert payload["settings"] == {"email": False, "inApp": True, "recipients": []}
    assert payload["userEmail"] == "me@firma.cz"


def test_get_view_notify_empty_string_when_email_unresolved(component, capsys, monkeypatch):
    monkeypatch.setattr(component.notify, "_query_me_email", lambda: None)
    component.handle_get("view=notify")
    assert _payload(capsys)["userEmail"] == ""


def test_post_save_notify_settings_persists(component, capsys):
    component.handle_post({
        "action": "save_notify_settings",
        "email": True,
        "inApp": False,
        "recipients": ["a@b.cz"],
    })
    payload = _payload(capsys)
    expected = {"email": True, "inApp": False, "recipients": ["a@b.cz"]}
    assert payload == {"ok": True, "settings": expected}
    assert component.notify_settings.get_notify() == expected


def test_post_save_notify_settings_drops_signed_in_user(component, capsys, monkeypatch):
    # The UI already blocks adding your own address, but the CLI/automation can
    # store it. The handler must strip it so it isn't persisted (and legacy dupes
    # self-heal on the next save).
    monkeypatch.setattr(component.notify, "_query_me_email", lambda: "me@firma.cz")
    component.handle_post({
        "action": "save_notify_settings",
        "email": True,
        "inApp": False,
        "recipients": ["ME@firma.cz", "a@x.cz"],
    })
    payload = _payload(capsys)
    assert payload["settings"]["recipients"] == ["a@x.cz"]
    assert component.notify_settings.get_notify()["recipients"] == ["a@x.cz"]


def test_post_save_notify_settings_absent_recipients_left_untouched(component, capsys, monkeypatch):
    # A save that omits `recipients` (e.g. toggling only the e-mail switch) must
    # not clear the stored list - self-stripping must not turn None into [].
    monkeypatch.setattr(component.notify, "_query_me_email", lambda: "me@firma.cz")
    component.handle_post({"action": "save_notify_settings", "recipients": ["a@b.cz"]})
    capsys.readouterr()
    component.handle_post({"action": "save_notify_settings", "email": True})
    payload = _payload(capsys)
    assert payload["settings"]["recipients"] == ["a@b.cz"]


def test_post_save_notify_settings_invalid_recipient_is_400(component, capsys):
    component.handle_post({
        "action": "save_notify_settings",
        "email": True,
        "inApp": True,
        "recipients": ["not-an-email"],
    })
    out = capsys.readouterr().out
    assert out.startswith("Status: 400 Bad Request")
    payload = json.loads(out.split("\r\n\r\n", 1)[1])
    assert payload["ok"] is False
    assert "error" in payload
    # rejected write must not persist
    assert component.notify_settings.get_notify()["recipients"] == []
