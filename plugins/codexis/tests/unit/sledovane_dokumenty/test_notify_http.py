"""HTTP-layer contract for the notify view/action in sledovane-cgi.

Loads the CGI handler by file path (it lives under a hyphenated component
directory, so it isn't an importable package) and drives its `handle_folder_*`
functions directly, capturing what would be emitted instead of going through
CGI stdout.
"""

import importlib.machinery
import importlib.util
import os

import pytest


def _load_http():
    path = os.path.join(
        os.path.dirname(__file__), "..", "..", "..", "components", "sledovane-dokumenty", "sledovane-cgi",
    )
    loader = importlib.machinery.SourceFileLoader("sledovane_http", path)
    spec = importlib.util.spec_from_file_location("sledovane_http", path, loader=loader)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_http = _load_http()


@pytest.fixture
def _isolated_home(dokumenty_app_dir, monkeypatch, tmp_path):
    """dokumenty_app_dir redirects state.APP_DIR; also point HOME at tmp so
    settings.defaults_path() (which reads the real ``~``) can't pick up
    anything left on the dev machine."""
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    return tmp_path


def _capture(monkeypatch):
    captured = {}

    def _fake_emit(payload, status=None):
        captured["payload"] = payload
        captured["status"] = status

    monkeypatch.setattr(_http, "emit_json", _fake_emit)
    return captured


def test_get_notify_includes_signed_in_user_email(_isolated_home, monkeypatch):
    monkeypatch.setattr(_http.notify, "_query_me_email", lambda: "me@firma.cz")
    captured = _capture(monkeypatch)

    _http.handle_folder_get("notify", {})

    assert captured["status"] is None
    assert captured["payload"]["mode"] == "notify"
    assert captured["payload"]["userEmail"] == "me@firma.cz"
    assert "settings" in captured["payload"]


def test_get_notify_user_email_empty_when_unresolved(_isolated_home, monkeypatch):
    monkeypatch.setattr(_http.notify, "_query_me_email", lambda: None)
    captured = _capture(monkeypatch)

    _http.handle_folder_get("notify", {})

    assert captured["payload"]["userEmail"] == ""


def test_save_notify_settings_invalid_recipient_returns_400(_isolated_home, monkeypatch):
    captured = _capture(monkeypatch)

    _http.handle_folder_action("save_notify_settings", {"recipients": ["not-an-email"]})

    assert captured["status"] == "400 Bad Request"
    assert captured["payload"]["ok"] is False
    assert "error" in captured["payload"]


def test_save_notify_settings_valid_recipients_still_ok(_isolated_home, monkeypatch):
    captured = _capture(monkeypatch)

    _http.handle_folder_action(
        "save_notify_settings",
        {"email": True, "inApp": False, "recipients": ["a@b.cz"]},
    )

    assert captured["status"] is None
    assert captured["payload"]["ok"] is True
    assert captured["payload"]["settings"]["recipients"] == ["a@b.cz"]
    assert captured["payload"]["settings"]["email"] is True
    assert captured["payload"]["settings"]["inApp"] is False


def test_save_notify_settings_drops_signed_in_user(_isolated_home, monkeypatch):
    # The UI already blocks adding your own address, but the CLI/automation can
    # store it. The handler must strip it so it isn't persisted (and legacy dupes
    # self-heal on the next save).
    monkeypatch.setattr(_http.notify, "_query_me_email", lambda: "me@firma.cz")
    captured = _capture(monkeypatch)

    _http.handle_folder_action(
        "save_notify_settings",
        {"email": True, "inApp": False, "recipients": ["ME@firma.cz", "a@x.cz"]},
    )

    assert captured["payload"]["settings"]["recipients"] == ["a@x.cz"]


def test_save_notify_settings_absent_recipients_left_untouched(_isolated_home, monkeypatch):
    # A save that omits `recipients` (e.g. toggling only the e-mail switch) must
    # not clear the stored list - self-stripping must not turn None into [].
    monkeypatch.setattr(_http.notify, "_query_me_email", lambda: "me@firma.cz")
    _http.settings.set_notify(email=True, inApp=True, recipients=["a@x.cz"])
    captured = _capture(monkeypatch)

    _http.handle_folder_action("save_notify_settings", {"email": False})

    assert captured["payload"]["settings"]["recipients"] == ["a@x.cz"]
