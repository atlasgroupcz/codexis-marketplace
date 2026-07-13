"""notify: summary building + send gating (no network — _send_email is stubbed)."""

import json
from unittest.mock import patch

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

    def test_no_email_when_no_changes(self):
        summary = notify.build_summary([_result("V-1/2026-701", change=None)])
        assert summary is None

    def test_no_email_when_only_errored_results(self):
        summary = notify.build_summary([_result("V-1/2026-701", ok=False, error="boom")])
        assert summary is None

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
    def test_no_changes_sends_nothing(self, monkeypatch):
        calls = []
        monkeypatch.setattr(notify, "_send_email", lambda *a, **k: calls.append((a, k)))
        n = notify.email_change_summary([_result("V-1/2026-701", change=None)])
        assert n == 0
        assert calls == []

    def test_empty_results_sends_nothing(self, monkeypatch):
        calls = []
        monkeypatch.setattr(notify, "_send_email", lambda *a, **k: calls.append((a, k)))
        assert notify.email_change_summary([]) == 0
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


# ── in-app (bell) notification ───────────────────────────────────────────────

_RESULTS_WITH_CHANGE = [_result("V-1/2026-701", change=_change())]
_RESULTS_NO_CHANGE = [_result("V-1/2026-701", change=None)]


class TestInappChangeSummary:
    def test_sends_subject_and_link_on_change(self, monkeypatch):
        captured = {}

        def fake_send_inapp(message, *, action=None, link=None):
            captured["message"] = message
            captured["action"] = action
            captured["link"] = link
            return True

        monkeypatch.setattr(notify, "_send_inapp", fake_send_inapp)

        sent = notify.inapp_change_summary(_RESULTS_WITH_CHANGE)

        assert sent is True
        expected_subject = notify._change_summary(notify._changed(_RESULTS_WITH_CHANGE))[0]
        assert captured["message"] == expected_subject
        assert captured["link"] == "/katastr"

    def test_no_change_does_not_send(self, monkeypatch):
        # No change: the bell stays silent — no _send_inapp call at all.
        calls = []
        monkeypatch.setattr(
            notify, "_send_inapp", lambda *a, **k: calls.append((a, k)) or True
        )

        sent = notify.inapp_change_summary(_RESULTS_NO_CHANGE)

        assert sent is False
        assert calls == []


# ── resolve_to (variant 2: signed-in user always included alongside extras) ──


class TestResolveTo:
    def test_resolve_to_empty_returns_none(self):
        assert notify.resolve_to([]) is None
        assert notify.resolve_to(None) is None

    def test_resolve_to_prepends_signed_in_user_when_extras(self):
        with patch.object(notify, "_query_me_email", return_value="me@firma.cz"):
            assert notify.resolve_to(["a@x.cz"]) == ["me@firma.cz", "a@x.cz"]

    def test_resolve_to_falls_back_to_extras_when_me_unavailable(self):
        with patch.object(notify, "_query_me_email", return_value=None):
            assert notify.resolve_to(["a@x.cz"]) == ["a@x.cz"]

    def test_resolve_to_dedupes_signed_in_user_case_insensitive(self):
        with patch.object(notify, "_query_me_email", return_value="me@x.cz"):
            assert notify.resolve_to(["ME@X.cz", "a@x.cz"]) == ["me@x.cz", "a@x.cz"]


# ── without_self (store-time guard: signed-in user is never an extra) ─────────


class TestWithoutSelf:
    def test_drops_signed_in_user_case_insensitive(self):
        with patch.object(notify, "_query_me_email", return_value="me@firma.cz"):
            assert notify.without_self(["ME@firma.CZ", "a@x.cz"]) == ["a@x.cz"]

    def test_keeps_others_untouched(self):
        with patch.object(notify, "_query_me_email", return_value="me@firma.cz"):
            assert notify.without_self(["a@x.cz", "b@y.cz"]) == ["a@x.cz", "b@y.cz"]

    def test_keeps_all_when_me_unavailable(self):
        with patch.object(notify, "_query_me_email", return_value=None):
            assert notify.without_self(["me@firma.cz", "a@x.cz"]) == ["me@firma.cz", "a@x.cz"]

    def test_empty_returns_empty_without_querying_me(self):
        # No extras → nothing to filter, so it must not make the GraphQL `me` call.
        with patch.object(
            notify, "_query_me_email", side_effect=AssertionError("must not query me")
        ):
            assert notify.without_self([]) == []
            assert notify.without_self(None) == []


class TestQueryMeEmail:
    def test_none_when_daemon_url_missing(self, monkeypatch, tmp_path):
        monkeypatch.delenv("CODEXIS_PUBLIC_DAEMON_URL", raising=False)
        monkeypatch.setenv("HOME", str(tmp_path))
        env_dir = tmp_path / ".cdx" / "env"
        env_dir.mkdir(parents=True, exist_ok=True)
        (env_dir / "secret").write_text("vm-token", encoding="utf-8")
        assert notify._query_me_email() is None

    def test_none_when_auth_missing(self, monkeypatch, tmp_path):
        # No ~/.cdx/env/secret written → _load_auth() returns None.
        monkeypatch.setenv("HOME", str(tmp_path))
        monkeypatch.setenv("CODEXIS_PUBLIC_DAEMON_URL", "http://daemon")
        assert notify._query_me_email() is None

    def test_parses_email_from_graphql_response(self, monkeypatch, tmp_path):
        cap = _stub_daemon(monkeypatch, tmp_path)

        class _Resp:
            def read(self):
                return json.dumps({"data": {"me": {"email": "me@firma.cz"}}}).encode("utf-8")

            def __enter__(self):
                return self

            def __exit__(self, *a):
                return False

        def fake_urlopen(req, timeout=0):
            cap["url"] = req.full_url
            cap["auth"] = req.headers.get("Authorization")
            return _Resp()

        monkeypatch.setattr(notify.urllib.request, "urlopen", fake_urlopen)
        assert notify._query_me_email() == "me@firma.cz"
        assert cap["url"] == "http://daemon"
        assert cap["auth"] == "Bearer vm-token"

    def test_none_on_transport_error(self, monkeypatch, tmp_path):
        _stub_daemon(monkeypatch, tmp_path)

        def fake_urlopen(req, timeout=0):
            raise OSError("boom")

        monkeypatch.setattr(notify.urllib.request, "urlopen", fake_urlopen)
        assert notify._query_me_email() is None

    @pytest.mark.parametrize("raw", [b"null", b"[]", b'"string"'])
    def test_none_on_non_dict_json_response(self, monkeypatch, tmp_path, raw):
        # A gateway/proxy in maintenance mode can return syntactically valid JSON
        # whose top level isn't an object (e.g. bare `null`, an array, or a string).
        # json.load succeeds, so this must not crash on the subsequent .get() — it
        # must return None.
        _stub_daemon(monkeypatch, tmp_path)

        class _Resp:
            def read(self):
                return raw

            def __enter__(self):
                return self

            def __exit__(self, *a):
                return False

        def fake_urlopen(req, timeout=0):
            return _Resp()

        monkeypatch.setattr(notify.urllib.request, "urlopen", fake_urlopen)
        assert notify._query_me_email() is None
