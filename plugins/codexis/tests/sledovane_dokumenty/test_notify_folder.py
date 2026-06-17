"""Notification settings + summary building for watched folders."""

from sledovane_dokumenty_core import notify


def _results(changes):
    return [{"root": "/r", "name": "Smlouvy", "changes": changes, "errors": []}]


def test_build_summary_none_without_changes():
    assert notify.build_summary(_results([])) is None
    assert notify.build_summary([]) is None


def test_build_summary_lists_changes():
    summary = notify.build_summary(
        _results([{"codexisId": "CR1", "text": "Občanský zákoník", "effective_on": "2026-07-01"}])
    )
    assert summary is not None
    subject, text, html = summary
    assert "Sledované dokumenty" in subject
    assert "Občanský zákoník" in text and "2026-07-01" in text
    assert "Občanský zákoník" in html and "Smlouvy" in html


def test_settings_default_and_roundtrip(dokumenty_app_dir):
    assert notify.load_settings() == {"email": True, "inApp": True, "recipients": []}
    saved = notify.save_settings({"email": False, "inApp": True})
    assert saved["email"] is False
    assert notify.load_settings()["email"] is False


def test_notify_changes_respects_settings(dokumenty_app_dir, monkeypatch):
    calls = {"email": 0, "inapp": 0}
    monkeypatch.setattr(
        notify, "_send_email",
        lambda *a, **k: calls.__setitem__("email", calls["email"] + 1) or True,
    )
    monkeypatch.setattr(
        notify, "_send_inapp",
        lambda *a, **k: calls.__setitem__("inapp", calls["inapp"] + 1) or True,
    )
    notify.save_settings({"email": True, "inApp": False})

    res = notify.notify_changes(_results([{"codexisId": "CR1", "text": "OZ"}]))
    assert calls == {"email": 1, "inapp": 0}
    assert res["changes"] == 1 and res["email"] is True and res["inApp"] is False


def test_notify_changes_noop_without_changes(dokumenty_app_dir, monkeypatch):
    monkeypatch.setattr(notify, "_send_email", lambda *a, **k: True)
    monkeypatch.setattr(notify, "_send_inapp", lambda *a, **k: True)
    res = notify.notify_changes(_results([]))
    assert res == {"email": False, "inApp": False, "changes": 0}
