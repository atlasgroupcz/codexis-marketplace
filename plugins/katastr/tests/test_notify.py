"""notify: summary building + send gating (no network — _send_email is stubbed)."""

import pytest

from katastr_core import notify


def _result(cislo, change=None, ok=True, error=None):
    return {"cislo_rizeni": cislo, "ok": ok, "change": change, "error": error}


def _change(old_stav="Podáno", new_stav="Povolení vkladu", ops=None, uhrada=False):
    return {
        "old_stav": old_stav,
        "new_stav": new_stav,
        "new_operations": ops or [],
        "stav_uhrady_changed": uhrada,
        "old_stav_uhrady": "N",
        "new_stav_uhrady": "U",
        "confirmed_on": None,
    }


def _stub_daemon(monkeypatch, tmp_path, daemon_url="http://daemon", secret: str | None = "vm-token"):
    """Point _send_email at a fake daemon: HOME→tmp, write ~/.cdx/env/secret, stub urlopen.

    Returns a dict populated with the outgoing request's url/auth/body.
    """
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("CODEXIS_PUBLIC_DAEMON_URL", daemon_url)
    if secret is not None:
        env_dir = tmp_path / ".cdx" / "env"
        env_dir.mkdir(parents=True, exist_ok=True)
        (env_dir / "secret").write_text(secret, encoding="utf-8")

    captured = {}

    class _Resp:
        def close(self):
            pass

    def fake_urlopen(req, timeout=0):
        captured["url"] = req.full_url
        captured["auth"] = req.headers.get("Authorization")
        captured["body"] = req.data.decode("utf-8")
        return _Resp()

    monkeypatch.setattr(notify.urllib.request, "urlopen", fake_urlopen)
    return captured


class TestBuildSummary:
    def test_none_only_when_no_results(self):
        assert notify.build_summary([]) is None

    def test_heartbeat_when_no_changes(self):
        summary = notify.build_summary([_result("V-1/2026-701", change=None)])
        assert summary is not None
        subject, text, html = summary
        assert "žádné nové změny" in subject.lower()
        assert "Zkontrolováno" in text

    def test_heartbeat_notes_errors(self):
        summary = notify.build_summary([_result("V-1/2026-701", ok=False, error="boom")])
        assert summary is not None
        _, text, _ = summary
        assert "nezdařila" in text

    def test_single_change_subject_and_body(self):
        r = _result(
            "V-1/2026-701",
            change=_change(ops=[{"nazev": "Zaplombování", "datumProvedeni": "2026-06-01"}]),
        )
        subject, text, html = notify.build_summary([r])
        assert "V-1/2026-701" in subject
        assert "Podáno → Povolení vkladu" in text
        assert "Zaplombování" in text
        assert "<li" in html

    def test_uhrada_change_uses_labels(self):
        r = _result("V-1/2026-701", change=_change(old_stav="x", new_stav="x", uhrada=True))
        _, text, _ = notify.build_summary([r])
        assert "Neuhrazeno → Uhrazeno" in text

    def test_multiple_changes_count_in_subject(self):
        rs = [_result("V-1/2026-701", change=_change()), _result("Z-2/2026-701", change=_change())]
        subject, _, _ = notify.build_summary(rs)
        assert "2" in subject


class TestEmailChangeSummary:
    def test_no_changes_sends_heartbeat(self, monkeypatch):
        calls = []
        monkeypatch.setattr(notify, "_send_email", lambda *a, **k: calls.append((a, k)))
        n = notify.email_change_summary([_result("V-1/2026-701", change=None)])
        assert n == 0
        assert len(calls) == 1

    def test_empty_results_sends_nothing(self, monkeypatch):
        calls = []
        monkeypatch.setattr(notify, "_send_email", lambda *a, **k: calls.append((a, k)))
        assert notify.email_change_summary([]) is None
        assert calls == []

    def test_sends_once_and_returns_count(self, monkeypatch):
        calls = []
        monkeypatch.setattr(notify, "_send_email", lambda *a, **k: calls.append((a, k)))
        n = notify.email_change_summary([
            _result("V-1/2026-701", change=_change()),
            _result("Z-2/2026-701", change=_change()),
            _result("P-3/2026-701", ok=False, error="boom"),
        ])
        assert n == 2
        assert len(calls) == 1


class TestRecipients:
    def test_to_forwarded_to_send_email(self, monkeypatch):
        captured = {}
        monkeypatch.setattr(notify, "_send_email", lambda *a, **k: captured.update(k))
        notify.email_change_summary(
            [_result("V-1/2026-701", change=_change())], to=["x@y.cz"]
        )
        assert captured.get("to") == ["x@y.cz"]

    def test_auth_uses_secret_file_not_env(self, monkeypatch, tmp_path):
        # CODEXIS_USER_API_TOKEN is a Codexis-DB JWT — must NOT authenticate the daemon API.
        monkeypatch.setenv("CODEXIS_USER_API_TOKEN", "codexis-token-must-not-be-used")
        cap = _stub_daemon(monkeypatch, tmp_path, secret="vm-token")
        notify._send_email("subj", "body")
        assert cap["auth"] == "Bearer vm-token"

    def test_secret_bearer_prefix_preserved(self, monkeypatch, tmp_path):
        cap = _stub_daemon(monkeypatch, tmp_path, secret="Bearer already-prefixed")
        notify._send_email("subj", "body")
        assert cap["auth"] == "Bearer already-prefixed"

    def test_missing_secret_raises(self, monkeypatch, tmp_path):
        monkeypatch.setenv("CODEXIS_USER_API_TOKEN", "codexis-token")  # present, still must fail
        _stub_daemon(monkeypatch, tmp_path, secret=None)
        with pytest.raises(notify.KatastrError):
            notify._send_email("subj", "body")

    def test_strips_graphql_from_daemon_url(self, monkeypatch, tmp_path):
        cap = _stub_daemon(monkeypatch, tmp_path, daemon_url="http://daemon:8090/graphql")
        notify._send_email("subj", "body")
        assert cap["url"] == "http://daemon:8090/rest/v1/plugin/email/send"

    def test_send_email_includes_to_in_payload(self, monkeypatch, tmp_path):
        cap = _stub_daemon(monkeypatch, tmp_path)
        notify._send_email("subj", "body", to=["x@y.cz"])
        assert '"to": ["x@y.cz"]' in cap["body"]

    def test_send_email_omits_to_by_default(self, monkeypatch, tmp_path):
        cap = _stub_daemon(monkeypatch, tmp_path)
        notify._send_email("subj", "body")
        assert '"to"' not in cap["body"]
