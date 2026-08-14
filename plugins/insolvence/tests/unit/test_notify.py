"""notify: daemon e-mail auth + transport (no network - urlopen is stubbed)."""

import json
from unittest.mock import patch

import pytest
from insolvence_core import notify

# A GraphQL answer the daemon accepted. It carries no `errors` key - that is the
# only thing that separates it from a rejection, which arrives as HTTP 200 too.
_ACCEPTED = {"data": {"createNotification": {"id": "n1"}}}


def _stub_daemon(
    monkeypatch, tmp_path, daemon_url="http://daemon", secret="vm-token", body=None
):
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
        """The daemon's real answer: a JSON body, read inside a `with`.

        The old stub offered only close(), which is why a mutation the daemon
        rejected still read as a delivery in these tests.
        """

        def read(self):
            payload = body if body is not None else _ACCEPTED
            # bytes pass through, so a test can hand over a body that is not JSON
            # at all - a proxy's HTML error page arrives as HTTP 200 too.
            if isinstance(payload, bytes):
                return payload
            return json.dumps(payload).encode("utf-8")

        def __enter__(self):
            return self

        def __exit__(self, *exc):
            return False

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
    # CODEXIS_USER_API_TOKEN is a Codexis-DB JWT - must NOT authenticate the daemon API.
    monkeypatch.setenv("CODEXIS_USER_API_TOKEN", "codexis-token-must-not-be-used")
    cap = _stub_daemon(monkeypatch, tmp_path, secret="vm-token")
    notify._send_email("subj", "body")
    assert cap["auth"] == "Bearer vm-token"


def test_secret_bearer_prefix_preserved(monkeypatch, tmp_path):
    cap = _stub_daemon(monkeypatch, tmp_path, secret="Bearer already-prefixed")
    notify._send_email("subj", "body")
    assert cap["auth"] == "Bearer already-prefixed"


def test_missing_secret_raises(monkeypatch, tmp_path):
    monkeypatch.setenv(
        "CODEXIS_USER_API_TOKEN", "codexis-token"
    )  # present, still must fail
    _stub_daemon(monkeypatch, tmp_path, secret=None)
    with pytest.raises(notify.InsolvenceError):
        notify._send_email("subj", "body")


def test_strips_graphql_from_daemon_url(monkeypatch, tmp_path):
    cap = _stub_daemon(monkeypatch, tmp_path, daemon_url="http://daemon:8090/graphql")
    notify._send_email("subj", "body")
    assert cap["url"] == "http://daemon:8090/rest/v1/plugin/email/send"


def test_send_email_includes_to_in_payload(monkeypatch, tmp_path):
    cap = _stub_daemon(monkeypatch, tmp_path)
    notify._send_email("subj", "body", to=["x@y.cz"])
    assert '"to": ["x@y.cz"]' in cap["body"]


def test_send_email_omits_to_by_default(monkeypatch, tmp_path):
    cap = _stub_daemon(monkeypatch, tmp_path)
    notify._send_email("subj", "body")
    assert '"to"' not in cap["body"]


def test_to_forwarded_to_send_email(monkeypatch):
    captured = {}
    monkeypatch.setattr(notify, "_send_email", lambda *a, **k: captured.update(k))
    notify.email_change_summary(_RESULTS_WITH_CHANGES, to=["x@y.cz"])
    assert captured.get("to") == ["x@y.cz"]


# ── in-app (bell) notification ───────────────────────────────────────────────

_RESULTS_WITH_CHANGES = [
    {
        "ok": True,
        "display_name": "Jan Novák",
        "changes": [{"popis": "Nové rozhodnutí soudu"}],
    },
]

_RESULTS_NO_CHANGES = [
    {"ok": True, "display_name": "Jan Novák", "changes": []},
]


def test_inapp_change_summary_sends_subject_and_link_on_changes(monkeypatch):
    captured = {}

    def fake_send_inapp(message, *, action=None, link=None):
        captured["message"] = message
        captured["action"] = action
        captured["link"] = link
        return True

    monkeypatch.setattr(notify, "_send_inapp", fake_send_inapp)
    # The bell link is resolved against the daemon, not hand-written; stub the
    # resolution so this test stays about what gets sent.
    monkeypatch.setattr(notify, "_resolve_app_link", lambda: "/apps/encoded-id")

    sent = notify.inapp_change_summary(_RESULTS_WITH_CHANGES)

    assert sent is True
    summary = notify.build_summary(_RESULTS_WITH_CHANGES)
    assert summary is not None
    expected_subject = summary[0]
    assert captured["message"] == expected_subject
    assert captured["link"] == "/apps/encoded-id"


def test_inapp_change_summary_no_changes_does_not_send(monkeypatch):
    calls = []
    monkeypatch.setattr(
        notify, "_send_inapp", lambda *a, **k: calls.append((a, k)) or True
    )

    sent = notify.inapp_change_summary(_RESULTS_NO_CHANGES)

    # None, not False: nothing was attempted. A quiet run must not read as a
    # failed delivery - False is reserved for a notification the daemon refused.
    assert sent is None
    assert calls == []


# ── resolve_to (variant 2: signed-in user always included alongside extras) ──


def test_resolve_to_empty_returns_none():
    assert notify.resolve_to([]) is None
    assert notify.resolve_to(None) is None


def test_resolve_to_prepends_signed_in_user_when_extras(monkeypatch):
    monkeypatch.setattr(notify, "_query_me_email", lambda: "me@firma.cz")
    assert notify.resolve_to(["a@x.cz"]) == ["me@firma.cz", "a@x.cz"]


def test_resolve_to_falls_back_to_extras_when_me_unavailable(monkeypatch):
    monkeypatch.setattr(notify, "_query_me_email", lambda: None)
    assert notify.resolve_to(["a@x.cz"]) == ["a@x.cz"]


def test_resolve_to_dedupes_signed_in_user_case_insensitive(monkeypatch):
    monkeypatch.setattr(notify, "_query_me_email", lambda: "me@x.cz")
    assert notify.resolve_to(["ME@X.cz", "a@x.cz"]) == ["me@x.cz", "a@x.cz"]


# ── without_self (store-time guard: signed-in user is never an extra) ─────────


def test_without_self_drops_signed_in_user_case_insensitive():
    with patch.object(notify, "_query_me_email", return_value="me@firma.cz"):
        assert notify.without_self(["ME@firma.CZ", "a@x.cz"]) == ["a@x.cz"]


def test_without_self_keeps_others_untouched():
    with patch.object(notify, "_query_me_email", return_value="me@firma.cz"):
        assert notify.without_self(["a@x.cz", "b@y.cz"]) == ["a@x.cz", "b@y.cz"]


def test_without_self_keeps_all_when_me_unavailable():
    with patch.object(notify, "_query_me_email", return_value=None):
        assert notify.without_self(["me@firma.cz", "a@x.cz"]) == [
            "me@firma.cz",
            "a@x.cz",
        ]


def test_without_self_empty_returns_empty_without_querying_me():
    # No extras → nothing to filter, so it must not make the GraphQL `me` call.
    with patch.object(
        notify, "_query_me_email", side_effect=AssertionError("must not query me")
    ):
        assert notify.without_self([]) == []
        assert notify.without_self(None) == []


# ── _query_me_email (best-effort GraphQL `me` lookup) ────────────────────────


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
            return json.dumps({"data": {"me": {"email": "me@firma.cz"}}}).encode(
                "utf-8"
            )

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
    # json.load succeeds, so this must not crash on the subsequent .get() - it
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


_EXPECTED_LINK_QUERY = ""
# ── app deep link ─────────────────────────────────────────────────────────────

# A real application id, captured from a running instance's /apps/... URL. It is
# only ever carried through, never parsed - these tests pin that.
_REAL_APP_ID = (
    "BlBsdWdpbkttYXJrZXRwbGFjZS1wbHVnaW46bWFya2V0cGxhY2UtZGNlODg2Y2MtMjhkZi0zNzk3"
    "LWFkY2UtYTU0OTY2OWE4NDY2OmthdGFzdHILQXBwbGljYXRpb24Ha2F0YXN0cg"
)


def _content_url(component_id):
    """The shape the daemon builds: prefix, plugin node id, component id, slash."""
    return (
        f"/rest/plugin-components/BlBsdWdpbkttYXJrZXRwbGFjZS1wbHVnaW4/{component_id}/"
    )


def _app(plugin=None, component=None, app_id=_REAL_APP_ID):
    return {
        "id": app_id,
        "contentUrl": _content_url(component or notify._COMPONENT_ID),
        "plugin": {"name": plugin or notify._PLUGIN_NAME},
    }


def test_reads_the_component_id_from_a_content_url():
    assert notify._component_id_of(_content_url("katastr")) == "katastr"


def test_reads_the_component_id_without_a_trailing_slash():
    assert notify._component_id_of("/rest/plugin-components/abc/znamky") == "znamky"


def test_has_no_component_id_for_a_degenerate_url():
    assert notify._component_id_of("") is None
    assert notify._component_id_of("/") is None
    assert notify._component_id_of(None) is None


def test_picks_the_application_matching_plugin_and_component():
    apps = [_app()]
    assert (
        notify._pick_application(apps, notify._PLUGIN_NAME, notify._COMPONENT_ID)
        == _REAL_APP_ID
    )


def test_ignores_an_application_of_another_plugin():
    # Same component id, different owning plugin - the plugin name is what keeps
    # two marketplaces' same-named components apart.
    apps = [_app(plugin="other-plugin")]
    assert (
        notify._pick_application(apps, notify._PLUGIN_NAME, notify._COMPONENT_ID)
        is None
    )


def test_ignores_a_sibling_component_of_the_same_plugin():
    apps = [_app(component="some-other-component")]
    assert (
        notify._pick_application(apps, notify._PLUGIN_NAME, notify._COMPONENT_ID)
        is None
    )


def test_returns_none_when_no_application_is_installed():
    assert (
        notify._pick_application([], notify._PLUGIN_NAME, notify._COMPONENT_ID) is None
    )


def test_skips_malformed_entries_instead_of_failing():
    apps = [
        {"plugin": {"name": notify._PLUGIN_NAME}},
        {"id": _REAL_APP_ID, "plugin": {"name": notify._PLUGIN_NAME}},
        {"id": _REAL_APP_ID, "contentUrl": _content_url(notify._COMPONENT_ID)},
        "not a dict",
        _app(),
    ]
    assert (
        notify._pick_application(apps, notify._PLUGIN_NAME, notify._COMPONENT_ID)
        == _REAL_APP_ID
    )


def _stub_applications_query(monkeypatch, tmp_path, payload):
    """Point the resolver at a fake daemon returning `payload`."""
    _stub_daemon(monkeypatch, tmp_path, daemon_url="http://daemon/graphql")

    class _Resp:
        def read(self):
            return json.dumps(payload).encode("utf-8")

        def __enter__(self):
            return self

        def __exit__(self, *exc):
            return False

    monkeypatch.setattr(
        notify.urllib.request, "urlopen", lambda req, timeout=0: _Resp()
    )


def test_resolve_app_link_builds_the_apps_route(monkeypatch, tmp_path):
    _stub_applications_query(
        monkeypatch, tmp_path, {"data": {"applications": [_app()]}}
    )
    assert notify._resolve_app_link() == f"/apps/{_REAL_APP_ID}{_EXPECTED_LINK_QUERY}"


def test_resolve_app_link_is_none_when_the_app_is_not_installed(monkeypatch, tmp_path):
    _stub_applications_query(monkeypatch, tmp_path, {"data": {"applications": []}})
    assert notify._resolve_app_link() is None


def test_resolve_app_link_is_none_without_daemon_config(monkeypatch, tmp_path):
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.delenv("CODEXIS_PUBLIC_DAEMON_URL", raising=False)
    assert notify._resolve_app_link() is None


# ── the daemon refused it ────────────────────────────────────────────────────
# HTTP 200 with the reason in the body is how a rejected mutation comes back, so
# a sender that only closes the response calls every rejection a delivery. That
# is what kept a dropped input field invisible for a month of scheduled runs.

_REJECTED = {
    "errors": [{"message": "Unknown field 'action' on type 'CreateNotificationInput'"}]
}


def test_inapp_reports_failure_when_the_daemon_rejects_the_mutation(
    monkeypatch, tmp_path
):
    _stub_daemon(monkeypatch, tmp_path, body=_REJECTED)

    assert notify._send_inapp("hello") is False


def test_inapp_reports_failure_when_the_body_is_not_json(monkeypatch, tmp_path):
    # A proxy's HTML error page: status 200, nothing json.load can read.
    _stub_daemon(monkeypatch, tmp_path, body=b"<html>502</html>")

    assert notify._send_inapp("hello") is False


def test_inapp_reports_success_when_the_daemon_accepts_the_mutation(
    monkeypatch, tmp_path
):
    _stub_daemon(monkeypatch, tmp_path)

    assert notify._send_inapp("hello") is True
