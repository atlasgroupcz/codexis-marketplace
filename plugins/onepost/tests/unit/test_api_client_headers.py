from onepost_core import settings
from onepost_core.api_client import OnepostClient


class FakeResponse:
    def __init__(self, payload: bytes):
        self.payload = payload

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        return False

    def read(self) -> bytes:
        return self.payload


def test_headers_include_default_daemon_origin(monkeypatch):
    monkeypatch.delenv("ONEPOST_API_ORIGIN", raising=False)
    monkeypatch.delenv("CODEXIS_PLUGIN_ONEPOST_API_ORIGIN", raising=False)
    monkeypatch.setattr(settings, "_read_env_file", lambda: {})

    client = OnepostClient(api_url="https://onepost.example", api_key="secret")

    assert client._headers()["Origin"] == settings.DEFAULT_DAEMON_ORIGIN


def test_headers_normalize_configured_daemon_origin(monkeypatch):
    monkeypatch.setenv("ONEPOST_API_ORIGIN", "https://vm.dev.internal.codexis.ai/apps/onepost")
    monkeypatch.setattr(settings, "_read_env_file", lambda: {})

    client = OnepostClient(api_url="https://onepost.example", api_key="secret")

    assert client._headers()["Origin"] == "https://vm.dev.internal.codexis.ai"


def test_upload_attachment_sends_multipart_request(monkeypatch, tmp_path):
    source = tmp_path / "note.txt"
    source.write_text("hello", encoding="utf-8")
    captured = {}

    def fake_urlopen(req, timeout):
        captured["req"] = req
        captured["timeout"] = timeout
        return FakeResponse(b'{"attachmentId":"att-1"}')

    monkeypatch.setattr("urllib.request.urlopen", fake_urlopen)

    client = OnepostClient(api_url="https://onepost.example", api_key="secret")
    response = client.upload_attachment(str(source))

    headers = {key.lower(): value for key, value in captured["req"].headers.items()}
    assert response == {"attachmentId": "att-1"}
    assert captured["req"].get_method() == "POST"
    assert captured["req"].full_url == "https://onepost.example/rest/api/v1/attachments"
    assert captured["timeout"] == 120
    assert headers["content-type"].startswith("multipart/form-data; boundary=")
    assert b'name="file"; filename="note.txt"' in captured["req"].data
    assert b"Content-Type: text/plain" in captured["req"].data
    assert b"hello" in captured["req"].data


def test_concept_helpers_use_expected_paths(monkeypatch):
    calls = []
    client = OnepostClient(api_url="https://onepost.example", api_key="secret")

    def fake_request(method, path, query=None, body=None, timeout=45):
        calls.append({"method": method, "path": path, "query": query, "body": body, "timeout": timeout})
        return {"ok": True}

    monkeypatch.setattr(client, "request", fake_request)

    save_response = client.save_concept({"mailboxId": "abc1234"})
    send_response = client.send_concept("CONCEPT-00000000-0000-0000-0000-000000000000")

    assert save_response == {"ok": True}
    assert send_response == {"ok": True}
    assert calls == [
        {"method": "POST", "path": "/concepts", "query": None, "body": {"mailboxId": "abc1234"}, "timeout": 45},
        {
            "method": "POST",
            "path": "/concepts/CONCEPT-00000000-0000-0000-0000-000000000000/send",
            "query": None,
            "body": None,
            "timeout": 300,
        },
    ]
