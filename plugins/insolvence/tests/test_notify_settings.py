import json

import pytest

from insolvence_core import notify_settings as ns


@pytest.fixture
def home(tmp_path, monkeypatch):
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))
    monkeypatch.setenv("HOME", str(tmp_path))
    d = tmp_path / ".local" / "share" / ns.APP_ID / "settings"
    d.mkdir(parents=True)
    (d / "defaults.json").write_text(json.dumps(ns.FALLBACK_DEFAULTS), encoding="utf-8")
    return tmp_path


def test_load_returns_defaults_when_no_user_file(home):
    assert ns.get_notify() == {"email": False, "inApp": True, "recipients": []}


def test_set_notify_writes_only_the_deviation(home):
    ns.set_notify(email=True)
    on_disk = json.loads((home / ".cdx" / "apps" / ns.APP_ID / "settings.json").read_text())
    assert on_disk["notify"] == {"email": True}
    assert ns.get_notify() == {"email": True, "inApp": True, "recipients": []}


def test_unset_keys_drift_with_defaults(home):
    ns.set_notify(email=True)
    d = home / ".local" / "share" / ns.APP_ID / "settings" / "defaults.json"
    data = json.loads(d.read_text())
    data["notify"]["inApp"] = False
    d.write_text(json.dumps(data), encoding="utf-8")
    assert ns.get_notify()["inApp"] is False


def test_ensure_settings_writes_minimal_sparse_file(home):
    ns.ensure_settings()
    on_disk = json.loads((home / ".cdx" / "apps" / ns.APP_ID / "settings.json").read_text())
    assert on_disk == {"version": 1}


def test_migrate_coerces_non_numeric_version(home):
    p = home / ".cdx" / "apps" / ns.APP_ID / "settings.json"
    p.parent.mkdir(parents=True)
    p.write_text(json.dumps({"version": "oops", "notify": {"email": True}}), encoding="utf-8")
    ns.ensure_settings()
    assert json.loads(p.read_text())["version"] == 1


def test_corrupt_user_file_falls_back(home):
    p = home / ".cdx" / "apps" / ns.APP_ID / "settings.json"
    p.parent.mkdir(parents=True)
    p.write_text("{ not json", encoding="utf-8")
    assert ns.get_notify() == {"email": False, "inApp": True, "recipients": []}


def test_clean_recipients_trims_dedups_preserves_order():
    out = ns.clean_recipients([" a@x.cz ", "b@y.cz", "A@X.CZ", ""])
    assert out == ["a@x.cz", "b@y.cz"]


def test_clean_recipients_rejects_invalid():
    with pytest.raises(ValueError):
        ns.clean_recipients(["nope"])


def test_set_notify_stores_cleaned_recipients(home):
    eff = ns.set_notify(recipients=["a@x.cz", "A@X.CZ", "b@y.cz"])
    assert eff["recipients"] == ["a@x.cz", "b@y.cz"]
