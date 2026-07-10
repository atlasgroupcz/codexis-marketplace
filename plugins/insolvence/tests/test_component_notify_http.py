"""HTTP CGI handler (components/insolvence/insolvence.py): notify settings.

Covers the uniform notify contract added on top of insolvence_core's
already-merged notify_settings/notify recipients backend:
- GET ?view=notify -> {mode, settings, userEmail}
- POST save_notify_settings -> {ok, settings} / {ok:false, error} on ValueError
"""

import importlib.util
import io
import json
from contextlib import redirect_stdout
from pathlib import Path

import pytest

COMPONENT_PATH = (
    Path(__file__).resolve().parents[1] / "components" / "insolvence" / "insolvence.py"
)


def _load_component():
    """Import insolvence.py fresh (it's a script, not a package)."""
    spec = importlib.util.spec_from_file_location("insolvence_component", COMPONENT_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _capture(fn, *args, **kwargs):
    buf = io.StringIO()
    with redirect_stdout(buf):
        fn(*args, **kwargs)
    header, _, body = buf.getvalue().partition("\r\n\r\n")
    return header, json.loads(body)


@pytest.fixture
def home(tmp_path, monkeypatch):
    """Isolate insolvence_core's notify_settings I/O to a fresh tmp home."""
    from insolvence_core import notify_settings as ns

    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    monkeypatch.setenv("HOME", str(tmp_path))
    d = tmp_path / ".local" / "share" / ns.APP_ID / "settings"
    d.mkdir(parents=True)
    (d / "defaults.json").write_text(json.dumps(ns.FALLBACK_DEFAULTS), encoding="utf-8")
    return tmp_path


@pytest.fixture
def cgi(home):
    return _load_component()


def test_module_imports(cgi):
    assert hasattr(cgi, "handle_get")
    assert hasattr(cgi, "handle_post")


def test_get_view_notify_shape_with_signed_in_email(cgi, monkeypatch):
    monkeypatch.setattr(cgi.notify, "_query_me_email", lambda: "me@firma.cz")
    _, body = _capture(cgi.handle_get, "view=notify")
    assert body == {
        "mode": "notify",
        "settings": {"email": False, "inApp": True, "recipients": []},
        "userEmail": "me@firma.cz",
    }


def test_get_view_notify_empty_email_when_unresolvable(cgi, monkeypatch):
    monkeypatch.setattr(cgi.notify, "_query_me_email", lambda: None)
    _, body = _capture(cgi.handle_get, "view=notify")
    assert body["userEmail"] == ""


def test_post_save_notify_settings_persists(cgi, monkeypatch):
    _, body = _capture(
        cgi.handle_post,
        {
            "action": "save_notify_settings",
            "email": True,
            "inApp": False,
            "recipients": ["a@x.cz", "A@X.CZ"],
        },
    )
    expected_settings = {"email": True, "inApp": False, "recipients": ["a@x.cz"]}
    assert body == {"ok": True, "settings": expected_settings}

    # Persisted: a fresh GET reflects the saved settings.
    monkeypatch.setattr(cgi.notify, "_query_me_email", lambda: None)
    _, body2 = _capture(cgi.handle_get, "view=notify")
    assert body2["settings"] == expected_settings


def test_post_save_notify_settings_invalid_recipient_is_400(cgi):
    header, body = _capture(
        cgi.handle_post,
        {"action": "save_notify_settings", "recipients": ["not-an-email"]},
    )
    assert header.startswith("Status: 400 Bad Request")
    assert body["ok"] is False
    assert "error" in body

    # Rejected write must not persist.
    from insolvence_core import notify_settings as ns

    assert ns.get_notify()["recipients"] == []
