import json
import os

import pytest
from sledovane_dokumenty_core import settings


def _write(path, obj):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(obj, f)


def test_load_returns_defaults_when_no_user_file(
    dokumenty_app_dir, monkeypatch, tmp_path
):
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))

    result = settings.load()

    assert result["notify"]["email"] is False
    assert result["notify"]["inApp"] is True
    assert result["notify"]["recipients"] == []


def test_user_override_wins_per_key(dokumenty_app_dir, monkeypatch, tmp_path):
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))
    _write(settings.user_path(), {"notify": {"email": True}})

    result = settings.load()

    assert result["notify"]["email"] is True  # overridden
    assert result["notify"]["inApp"] is True  # from defaults
    assert result["notify"]["recipients"] == []  # from defaults


def test_load_defaults_falls_back_when_file_missing(monkeypatch, tmp_path):
    monkeypatch.setattr(settings, "defaults_path", lambda: str(tmp_path / "nope.json"))
    assert settings.load_defaults() == settings.FALLBACK_DEFAULTS


def test_load_falls_back_when_user_file_corrupt(
    dokumenty_app_dir, monkeypatch, tmp_path
):
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))

    user_file = settings.user_path()
    os.makedirs(os.path.dirname(user_file), exist_ok=True)
    with open(user_file, "wb") as f:
        f.write(b"\xff\xff not json")

    result = settings.load()

    assert result["notify"]["email"] is False
    assert result["notify"]["inApp"] is True
    assert result["notify"]["recipients"] == []


def test_ensure_materialises_full_file_on_first_run(
    dokumenty_app_dir, monkeypatch, tmp_path
):
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))

    assert not os.path.exists(settings.user_path())
    settings.ensure_settings()

    written = json.load(open(settings.user_path(), encoding="utf-8"))
    assert written == {"version": 1}  # sparse: no defaults materialised


def test_ensure_does_not_freeze_user_out_of_new_default_values(
    dokumenty_app_dir, monkeypatch, tmp_path
):
    """A changed default must reach a key the user never set (no reconcile-freeze)."""
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": False, "recipients": []}},
    )  # inApp now False
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))
    _write(
        settings.user_path(), {"version": 1, "notify": {"email": True}}
    )  # existing user choice

    settings.ensure_settings()

    written = json.load(open(settings.user_path(), encoding="utf-8"))
    assert written == {
        "version": 1,
        "notify": {"email": True},
    }  # untouched, still sparse

    effective = settings.load()["notify"]
    assert effective == {"email": True, "inApp": False, "recipients": []}


def test_ensure_does_not_write_when_nothing_changed(
    dokumenty_app_dir, monkeypatch, tmp_path
):
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))
    settings.ensure_settings()  # first run writes
    mtime = os.path.getmtime(settings.user_path())

    settings.ensure_settings()  # second run: nothing changed

    assert os.path.getmtime(settings.user_path()) == mtime  # no rewrite


def test_ensure_then_default_change_does_not_freeze_existing_users(
    dokumenty_app_dir, monkeypatch, tmp_path
):
    """End-to-end: first run writes sparse file; a later default flip reaches the user."""
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))

    assert not os.path.exists(settings.user_path())
    settings.ensure_settings()
    written = json.load(open(settings.user_path(), encoding="utf-8"))
    assert written == {"version": 1}

    # A new version ships with a flipped default -- the user never opted into inApp.
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": False, "recipients": []}},
    )

    assert settings.load()["notify"]["inApp"] is False  # new default reaches the user


def test_set_notify_writes_only_the_deviation(dokumenty_app_dir, monkeypatch, tmp_path):
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))

    settings.ensure_settings()  # as the real CLI does before any set_notify call
    settings.set_notify(email=True)

    raw = json.load(open(settings.user_path(), encoding="utf-8"))
    assert raw == {
        "version": 1,
        "notify": {"email": True},
    }  # sparse: only version + deviation
    assert settings.get_notify()["email"] is True  # effective reflects it
    assert settings.get_notify()["inApp"] is True  # still from defaults


def test_set_notify_writes_recipients_sparse(dokumenty_app_dir, monkeypatch, tmp_path):
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))

    settings.ensure_settings()  # as the real CLI does before any set_notify call
    settings.set_notify(recipients=["a@b.cz"])

    raw = json.load(open(settings.user_path(), encoding="utf-8"))
    assert raw == {
        "version": 1,
        "notify": {"recipients": ["a@b.cz"]},
    }  # sparse: only version + deviation
    assert settings.get_notify()["recipients"] == ["a@b.cz"]


def test_clean_recipients_trims_dedups_preserves_order():
    out = settings.clean_recipients([" a@x.cz ", "b@y.cz", "A@X.CZ", ""])
    assert out == ["a@x.cz", "b@y.cz"]


def test_clean_recipients_rejects_invalid():
    with pytest.raises(ValueError):
        settings.clean_recipients(["nope"])


def test_set_notify_stores_cleaned_recipients(dokumenty_app_dir, monkeypatch, tmp_path):
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))

    eff = settings.set_notify(recipients=["a@x.cz", "A@X.CZ", "b@y.cz"])

    assert eff["recipients"] == ["a@x.cz", "b@y.cz"]


def test_set_notify_rejects_invalid_recipient(dokumenty_app_dir, monkeypatch, tmp_path):
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))

    with pytest.raises(ValueError):
        settings.set_notify(recipients=["nope"])


def test_migrates_legacy_folder_watch_settings(
    dokumenty_app_dir, monkeypatch, tmp_path
):
    shipped = tmp_path / "defaults.json"
    _write(
        str(shipped),
        {"version": 1, "notify": {"email": False, "inApp": True, "recipients": []}},
    )
    monkeypatch.setattr(settings, "defaults_path", lambda: str(shipped))
    legacy = os.path.join(str(dokumenty_app_dir), "folder-watch-settings.json")
    _write(legacy, {"email": True, "inApp": False, "recipients": ["a@b.cz"]})

    settings.ensure_settings()

    assert not os.path.exists(legacy)  # old file removed
    eff = settings.get_notify()
    assert (
        eff["email"] is True
        and eff["inApp"] is False
        and eff["recipients"] == ["a@b.cz"]
    )
