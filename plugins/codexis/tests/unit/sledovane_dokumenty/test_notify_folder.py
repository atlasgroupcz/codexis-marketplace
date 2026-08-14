"""Notification settings + summary building for watched folders."""

import json

import pytest
from sledovane_dokumenty_core import notify
from sledovane_dokumenty_core import settings as settings_mod


def _results(changes):
    return [{"root": "/r", "name": "Smlouvy", "changes": changes, "errors": []}]


def test_build_summary_none_without_changes():
    assert notify.build_summary(_results([])) is None
    assert notify.build_summary([]) is None


def test_build_summary_lists_changes():
    summary = notify.build_summary(
        _results(
            [
                {
                    "codexisId": "CR1",
                    "text": "Občanský zákoník",
                    "effective_on": "2026-07-01",
                }
            ]
        )
    )
    assert summary is not None
    subject, text, html = summary
    assert "Sledované dokumenty" in subject
    assert "Občanský zákoník" in text and "2026-07-01" in text
    assert "Občanský zákoník" in html and "Smlouvy" in html


def test_notify_changes_respects_settings(dokumenty_app_dir, monkeypatch):
    calls = {"email": 0, "inapp": 0}
    monkeypatch.setattr(
        notify,
        "_send_email",
        lambda *a, **k: calls.__setitem__("email", calls["email"] + 1) or True,
    )
    monkeypatch.setattr(
        notify,
        "_send_inapp",
        lambda *a, **k: calls.__setitem__("inapp", calls["inapp"] + 1) or True,
    )
    monkeypatch.setattr(
        settings_mod,
        "get_notify",
        lambda: {"email": True, "inApp": False, "recipients": []},
    )

    res = notify.notify_changes(_results([{"codexisId": "CR1", "text": "OZ"}]))
    assert calls == {"email": 1, "inapp": 0}
    assert res["changes"] == 1 and res["email"] is True and res["inApp"] is False


def test_notify_changes_noop_without_changes(dokumenty_app_dir, monkeypatch):
    monkeypatch.setattr(notify, "_send_email", lambda *a, **k: True)
    monkeypatch.setattr(notify, "_send_inapp", lambda *a, **k: True)
    res = notify.notify_changes(_results([]))
    assert res == {"email": False, "inApp": False, "changes": 0}


def _stub_daemon(monkeypatch, tmp_path, daemon_url="http://daemon", secret="vm-token"):
    """HOME→tmp + ~/.cdx/env/secret + stubbed urlopen; returns captured url/auth."""
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
        return _Resp()

    monkeypatch.setattr(notify.urllib.request, "urlopen", fake_urlopen)
    return captured


def test_email_auth_uses_secret_not_env(monkeypatch, tmp_path):
    # CODEXIS_USER_API_TOKEN is a Codexis-DB JWT - must NOT authenticate the daemon API.
    monkeypatch.setenv("CODEXIS_USER_API_TOKEN", "codexis-token-must-not-be-used")
    cap = _stub_daemon(monkeypatch, tmp_path, daemon_url="http://daemon:8090/graphql")
    assert notify._send_email("subj", "body", None) is True
    assert cap["auth"] == "Bearer vm-token"
    # GraphQL URL normalized to the REST base for the plugin e-mail endpoint.
    assert cap["url"] == "http://daemon:8090/rest/v1/plugin/email/send"


def test_inapp_auth_uses_secret_not_env(monkeypatch, tmp_path):
    monkeypatch.setenv("CODEXIS_USER_API_TOKEN", "codexis-token-must-not-be-used")
    cap = _stub_daemon(monkeypatch, tmp_path, daemon_url="http://daemon")
    assert notify._send_inapp("hello") is True
    assert cap["auth"] == "Bearer vm-token"
    assert cap["url"] == "http://daemon/graphql"


def test_email_returns_false_without_secret(monkeypatch, tmp_path):
    monkeypatch.setenv(
        "CODEXIS_USER_API_TOKEN", "codexis-token"
    )  # present, still must fail
    _stub_daemon(monkeypatch, tmp_path, secret=None)
    assert notify._send_email("subj", "body", None) is False


# Folder results shape that build_summary/_changed_folders already consume
# (see the existing _results() helper at the top of this test file).
def _folder_results():
    return [
        {
            "root": "/r",
            "name": "Smlouvy",
            "errors": [],
            "changes": [
                {
                    "codexisId": "CR1",
                    "text": "Občanský zákoník",
                    "effective_on": "2026-07-01",
                }
            ],
        }
    ]


def test_notify_changes_reads_unified_setting(dokumenty_app_dir, monkeypatch):
    calls = {"email": 0, "inapp": 0}
    monkeypatch.setattr(
        notify,
        "_send_email",
        lambda *a, **k: calls.__setitem__("email", calls["email"] + 1) or True,
    )
    monkeypatch.setattr(
        notify,
        "_send_inapp",
        lambda *a, **k: calls.__setitem__("inapp", calls["inapp"] + 1) or True,
    )
    monkeypatch.setattr(
        settings_mod,
        "get_notify",
        lambda: {"email": True, "inApp": False, "recipients": []},
    )

    notify.notify_changes(
        _folder_results()
    )  # settings=None -> reads the unified setting

    assert calls["email"] == 1
    assert calls["inapp"] == 0


def test_notify_document_changes_respects_setting(dokumenty_app_dir, monkeypatch):
    calls = {"email": 0, "inapp": 0}
    monkeypatch.setattr(
        notify,
        "_send_email",
        lambda *a, **k: calls.__setitem__("email", calls["email"] + 1) or True,
    )
    monkeypatch.setattr(
        notify,
        "_send_inapp",
        lambda *a, **k: calls.__setitem__("inapp", calls["inapp"] + 1) or True,
    )
    monkeypatch.setattr(
        settings_mod,
        "get_notify",
        lambda: {"email": True, "inApp": True, "recipients": []},
    )

    changed = [{"codexisId": "CR1", "name": "Zákoník práce"}]
    out = notify.notify_document_changes(changed)

    assert out["changes"] == 1
    assert calls["email"] == 1 and calls["inapp"] == 1


def test_notify_document_changes_noop_when_empty(dokumenty_app_dir, monkeypatch):
    monkeypatch.setattr(
        settings_mod,
        "get_notify",
        lambda: {"email": True, "inApp": True, "recipients": []},
    )
    assert notify.notify_document_changes([])["changes"] == 0
    assert notify.build_document_summary([]) is None


# ── resolve_to (signed-in user always included alongside extras) ────────────


class TestResolveTo:
    def test_empty_returns_none(self):
        assert notify.resolve_to([]) is None
        assert notify.resolve_to(None) is None

    def test_prepends_signed_in_user_when_extras(self, monkeypatch):
        monkeypatch.setattr(notify, "_query_me_email", lambda: "me@firma.cz")
        assert notify.resolve_to(["a@x.cz"]) == ["me@firma.cz", "a@x.cz"]

    def test_falls_back_to_extras_when_me_unavailable(self, monkeypatch):
        monkeypatch.setattr(notify, "_query_me_email", lambda: None)
        assert notify.resolve_to(["a@x.cz"]) == ["a@x.cz"]

    def test_dedupes_signed_in_user_case_insensitively(self, monkeypatch):
        monkeypatch.setattr(notify, "_query_me_email", lambda: "me@x.cz")
        assert notify.resolve_to(["ME@X.cz", "a@x.cz"]) == ["me@x.cz", "a@x.cz"]


# ── without_self (store-time guard: signed-in user is never an extra) ─────────


class TestWithoutSelf:
    def test_drops_signed_in_user_case_insensitive(self, monkeypatch):
        monkeypatch.setattr(notify, "_query_me_email", lambda: "me@firma.cz")
        assert notify.without_self(["ME@firma.CZ", "a@x.cz"]) == ["a@x.cz"]

    def test_keeps_others_untouched(self, monkeypatch):
        monkeypatch.setattr(notify, "_query_me_email", lambda: "me@firma.cz")
        assert notify.without_self(["a@x.cz", "b@y.cz"]) == ["a@x.cz", "b@y.cz"]

    def test_keeps_all_when_me_unavailable(self, monkeypatch):
        monkeypatch.setattr(notify, "_query_me_email", lambda: None)
        assert notify.without_self(["me@firma.cz", "a@x.cz"]) == [
            "me@firma.cz",
            "a@x.cz",
        ]

    def test_empty_returns_empty_without_querying_me(self, monkeypatch):
        # No extras → nothing to filter, so it must not make the GraphQL `me` call.
        def _boom():
            raise AssertionError("must not query me")

        monkeypatch.setattr(notify, "_query_me_email", _boom)
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
        # No ~/.cdx/env/secret written -> load_auth() returns None.
        monkeypatch.setenv("HOME", str(tmp_path))
        monkeypatch.setenv("CODEXIS_PUBLIC_DAEMON_URL", "http://daemon")
        assert notify._query_me_email() is None

    def test_parses_email_from_graphql_response(self, monkeypatch, tmp_path):
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
        assert cap["url"] == "http://daemon/graphql"
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
        # json.load succeeds, so this must not crash on the subsequent .get() -- it
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


# ── recipients wiring: resolve_to() feeds _send_email(recipients=...) ───────


def test_notify_changes_forwards_resolved_recipients(dokumenty_app_dir, monkeypatch):
    captured = {}
    monkeypatch.setattr(
        notify, "_send_email", lambda *a, **k: captured.update(k) or True
    )
    monkeypatch.setattr(notify, "_send_inapp", lambda *a, **k: True)
    monkeypatch.setattr(notify, "resolve_to", lambda recipients: ["resolved@x.cz"])
    monkeypatch.setattr(
        settings_mod,
        "get_notify",
        lambda: {"email": True, "inApp": False, "recipients": ["a@x.cz"]},
    )

    notify.notify_changes(_folder_results())

    assert captured.get("recipients") == ["resolved@x.cz"]


def test_notify_document_changes_forwards_resolved_recipients(
    dokumenty_app_dir, monkeypatch
):
    captured = {}
    monkeypatch.setattr(
        notify, "_send_email", lambda *a, **k: captured.update(k) or True
    )
    monkeypatch.setattr(notify, "_send_inapp", lambda *a, **k: True)
    monkeypatch.setattr(notify, "resolve_to", lambda recipients: ["resolved@x.cz"])
    monkeypatch.setattr(
        settings_mod,
        "get_notify",
        lambda: {"email": True, "inApp": False, "recipients": ["a@x.cz"]},
    )

    notify.notify_document_changes([{"codexisId": "CR1", "name": "Zákoník práce"}])

    assert captured.get("recipients") == ["resolved@x.cz"]


_EXPECTED_LINK_QUERY = "?tab=folders"
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


def test_change_notification_sends_no_action_field(dokumenty_app_dir, monkeypatch):
    # `CreateNotificationInput` has no `action` - the daemon dropped the
    # call-to-action button, so a caller that still passes a label has the whole
    # mutation rejected and the notification never arrives.
    sent = []
    monkeypatch.setattr(notify, "_send_inapp", lambda *a, **k: sent.append(k) or True)
    monkeypatch.setattr(
        settings_mod,
        "get_notify",
        lambda: {"email": False, "inApp": True, "recipients": []},
    )

    notify.notify_changes(_folder_results())

    assert sent, "the in-app notification was not attempted"
    assert "action" not in sent[0], f"still sends a dropped field: {sent[0]}"
