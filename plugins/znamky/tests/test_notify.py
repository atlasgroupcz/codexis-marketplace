"""notify: daemon e-mail auth + transport (no network — urlopen is stubbed)."""

import json
from unittest.mock import patch

import pytest

from znamky_core import notify


def _stub_daemon(monkeypatch, tmp_path, daemon_url="http://daemon", secret="vm-token"):
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


def test_auth_uses_secret_file_not_env(monkeypatch, tmp_path):
    # CODEXIS_USER_API_TOKEN is a Codexis-DB JWT — must NOT authenticate the daemon API.
    monkeypatch.setenv("CODEXIS_USER_API_TOKEN", "codexis-token-must-not-be-used")
    cap = _stub_daemon(monkeypatch, tmp_path, secret="vm-token")
    notify._send_email("subj", "body")
    assert cap["auth"] == "Bearer vm-token"


def test_secret_bearer_prefix_preserved(monkeypatch, tmp_path):
    cap = _stub_daemon(monkeypatch, tmp_path, secret="Bearer already-prefixed")
    notify._send_email("subj", "body")
    assert cap["auth"] == "Bearer already-prefixed"


def test_missing_secret_raises(monkeypatch, tmp_path):
    monkeypatch.setenv("CODEXIS_USER_API_TOKEN", "codexis-token")  # present, still must fail
    _stub_daemon(monkeypatch, tmp_path, secret=None)
    with pytest.raises(notify.ZnamkyError):
        notify._send_email("subj", "body")


def test_strips_graphql_from_daemon_url(monkeypatch, tmp_path):
    cap = _stub_daemon(monkeypatch, tmp_path, daemon_url="http://daemon:8090/graphql")
    notify._send_email("subj", "body")
    assert cap["url"] == "http://daemon:8090/rest/v1/plugin/email/send"


# ── recipients (`to`) ─────────────────────────────────────────────────────────

_RESULTS_ONE_CHANGE = [
    {
        "ok": True,
        "display_name": "Test Mark",
        "new_collisions": [
            {"mark_text": "Podobná známka", "tier": "high", "applicant": "Foo s.r.o."}
        ],
    },
]


def test_to_forwarded_to_send_email(monkeypatch):
    captured = {}
    monkeypatch.setattr(notify, "_send_email", lambda *a, **k: captured.update(k))
    notify.email_change_summary(_RESULTS_ONE_CHANGE, to=["x@y.cz"])
    assert captured.get("to") == ["x@y.cz"]


def test_send_email_includes_to_in_payload(monkeypatch, tmp_path):
    cap = _stub_daemon(monkeypatch, tmp_path)
    notify._send_email("subj", "body", to=["x@y.cz"])
    assert '"to": ["x@y.cz"]' in cap["body"]


def test_send_email_omits_to_by_default(monkeypatch, tmp_path):
    cap = _stub_daemon(monkeypatch, tmp_path)
    notify._send_email("subj", "body")
    assert '"to"' not in cap["body"]


# ── resolve_to (variant 2: signed-in user always included alongside extras) ──


def test_resolve_to_empty_returns_none():
    assert notify.resolve_to([]) is None
    assert notify.resolve_to(None) is None


def test_resolve_to_prepends_signed_in_user_when_extras():
    with patch.object(notify, "_query_me_email", return_value="me@firma.cz"):
        assert notify.resolve_to(["a@x.cz"]) == ["me@firma.cz", "a@x.cz"]


def test_resolve_to_falls_back_to_extras_when_me_unavailable():
    with patch.object(notify, "_query_me_email", return_value=None):
        assert notify.resolve_to(["a@x.cz"]) == ["a@x.cz"]


def test_resolve_to_dedupes_signed_in_user_case_insensitive():
    with patch.object(notify, "_query_me_email", return_value="me@x.cz"):
        assert notify.resolve_to(["ME@X.cz", "a@x.cz"]) == ["me@x.cz", "a@x.cz"]


# ── _query_me_email ────────────────────────────────────────────────────────────


def test_query_me_email_none_when_daemon_url_missing(monkeypatch, tmp_path):
    monkeypatch.delenv("CODEXIS_PUBLIC_DAEMON_URL", raising=False)
    monkeypatch.setenv("HOME", str(tmp_path))
    env_dir = tmp_path / ".cdx" / "env"
    env_dir.mkdir(parents=True, exist_ok=True)
    (env_dir / "secret").write_text("vm-token", encoding="utf-8")
    assert notify._query_me_email() is None


def test_query_me_email_none_when_auth_missing(monkeypatch, tmp_path):
    # No ~/.cdx/env/secret written → _load_auth() returns None.
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("CODEXIS_PUBLIC_DAEMON_URL", "http://daemon")
    assert notify._query_me_email() is None


def test_query_me_email_parses_email_from_graphql_response(monkeypatch, tmp_path):
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


def test_query_me_email_none_on_transport_error(monkeypatch, tmp_path):
    _stub_daemon(monkeypatch, tmp_path)

    def fake_urlopen(req, timeout=0):
        raise OSError("boom")

    monkeypatch.setattr(notify.urllib.request, "urlopen", fake_urlopen)
    assert notify._query_me_email() is None


@pytest.mark.parametrize("raw", [b"null", b"[]", b'"string"'])
def test_query_me_email_none_on_non_dict_json_response(monkeypatch, tmp_path, raw):
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


# ── in-app (bell) notification ───────────────────────────────────────────────

_RESULTS_WITH_CHANGES = [
    {
        "ok": True,
        "display_name": "Test Mark",
        "new_collisions": [
            {"mark_text": "Podobná známka", "tier": "high", "applicant": "Foo s.r.o."}
        ],
    },
]

_RESULTS_NO_CHANGES = [
    {"ok": True, "display_name": "Test Mark", "new_collisions": []},
]


def test_inapp_change_summary_sends_subject_and_link_on_changes(monkeypatch):
    captured = {}

    def fake_send_inapp(message, *, action=None, link=None):
        captured["message"] = message
        captured["action"] = action
        captured["link"] = link
        return True

    monkeypatch.setattr(notify, "_send_inapp", fake_send_inapp)

    sent = notify.inapp_change_summary(_RESULTS_WITH_CHANGES)

    assert sent is True
    expected_subject = notify.build_summary(_RESULTS_WITH_CHANGES)[0]
    assert captured["message"] == expected_subject
    assert captured["link"] == "/znamky"


def test_inapp_change_summary_no_changes_does_not_send(monkeypatch):
    calls = []
    monkeypatch.setattr(
        notify, "_send_inapp", lambda *a, **k: calls.append((a, k)) or True
    )

    sent = notify.inapp_change_summary(_RESULTS_NO_CHANGES)

    assert sent is False
    assert calls == []
