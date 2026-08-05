"""notify_settings: shipped defaults + sparse user override + recipients validation."""

import json
import os

import pytest
from sledovana_judikatura_core import notify_settings as ns


def _write(path, obj):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(obj, f)


@pytest.fixture
def defaults(judikatura_app_dir, monkeypatch, tmp_path):
    shipped = tmp_path / "defaults.json"
    _write(str(shipped), ns.FALLBACK_DEFAULTS)
    monkeypatch.setattr(ns, "defaults_path", lambda: str(shipped))
    return shipped


def test_load_returns_defaults_when_no_user_file(defaults):
    assert ns.get_notify() == {"email": False, "inApp": True, "recipients": []}


def test_set_notify_writes_only_the_deviation(defaults, judikatura_app_dir):
    ns.set_notify(email=True)
    on_disk = json.loads((judikatura_app_dir / "settings.json").read_text())
    assert on_disk["notify"] == {"email": True}
    assert ns.get_notify() == {"email": True, "inApp": True, "recipients": []}


def test_unset_keys_drift_with_defaults(defaults):
    ns.set_notify(email=True)
    data = json.loads(defaults.read_text())
    data["notify"]["inApp"] = False
    defaults.write_text(json.dumps(data), encoding="utf-8")
    assert ns.get_notify()["inApp"] is False


def test_load_defaults_falls_back_when_file_missing(monkeypatch, tmp_path):
    monkeypatch.setattr(ns, "defaults_path", lambda: str(tmp_path / "nope.json"))
    assert ns.load_defaults() == ns.FALLBACK_DEFAULTS


def test_ensure_settings_writes_minimal_sparse_file(defaults, judikatura_app_dir):
    ns.ensure_settings()
    on_disk = json.loads((judikatura_app_dir / "settings.json").read_text())
    assert on_disk == {"version": 1}


def test_ensure_does_not_write_when_nothing_changed(defaults, judikatura_app_dir):
    ns.ensure_settings()
    p = judikatura_app_dir / "settings.json"
    mtime = os.path.getmtime(p)
    ns.ensure_settings()
    assert os.path.getmtime(p) == mtime


def test_migrate_coerces_non_numeric_version(defaults, judikatura_app_dir):
    p = judikatura_app_dir / "settings.json"
    p.write_text(
        json.dumps({"version": "oops", "notify": {"email": True}}), encoding="utf-8"
    )
    ns.ensure_settings()
    assert json.loads(p.read_text())["version"] == 1


def test_corrupt_user_file_falls_back(defaults, judikatura_app_dir):
    p = judikatura_app_dir / "settings.json"
    p.write_text("{ not json", encoding="utf-8")
    assert ns.get_notify() == {"email": False, "inApp": True, "recipients": []}


def test_clean_recipients_trims_dedups_preserves_order():
    out = ns.clean_recipients([" a@x.cz ", "b@y.cz", "A@X.CZ", ""])
    assert out == ["a@x.cz", "b@y.cz"]


def test_clean_recipients_rejects_invalid():
    with pytest.raises(ValueError):
        ns.clean_recipients(["nope"])


def test_set_notify_stores_cleaned_recipients(defaults):
    eff = ns.set_notify(recipients=["a@x.cz", "A@X.CZ", "b@y.cz"])
    assert eff["recipients"] == ["a@x.cz", "b@y.cz"]
