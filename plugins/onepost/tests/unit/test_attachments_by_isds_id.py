from onepost_core.api_client import OnepostClient
from onepost_core.exceptions import ApiHttpError, ApiPermissionError

ISDS_ID = "12054584"
INTERNAL_ID = "msg-uuid-1"
ATTACHMENTS = [{"id": "att-1", "fileName": "rozhodnuti.pdf"}]


def build_client() -> OnepostClient:
    return OnepostClient(api_url="https://onepost.example", api_key="secret")


def install_request(monkeypatch, client, handler) -> list[str]:
    paths: list[str] = []

    def fake_request(method, path, query=None, body=None, timeout=45):
        paths.append(path)
        return handler(path)

    monkeypatch.setattr(client, "request", fake_request)
    return paths


def test_internal_id_fetches_attachments_without_a_lookup(monkeypatch):
    client = build_client()
    paths = install_request(monkeypatch, client, lambda path: ATTACHMENTS)

    assert client.get_attachments(INTERNAL_ID) == ATTACHMENTS
    assert paths == [f"/messages/{INTERNAL_ID}/attachments"]


def test_isds_id_retries_with_the_internal_id_the_message_reports(monkeypatch):
    client = build_client()

    def handler(path):
        if path == f"/messages/{ISDS_ID}/attachments":
            raise ApiHttpError(404, "ONEPOST API vrátilo HTTP_ERROR (HTTP 404)")
        if path == f"/messages/{ISDS_ID}":
            return {"id": INTERNAL_ID, "isdsMessageId": ISDS_ID, "attachmentCount": 1}
        if path == f"/messages/{INTERNAL_ID}/attachments":
            return ATTACHMENTS
        raise AssertionError(f"unexpected path {path}")

    paths = install_request(monkeypatch, client, handler)

    assert client.get_attachments(ISDS_ID) == ATTACHMENTS
    assert paths == [
        f"/messages/{ISDS_ID}/attachments",
        f"/messages/{ISDS_ID}",
        f"/messages/{INTERNAL_ID}/attachments",
    ]


def test_unknown_id_keeps_the_original_not_found_error(monkeypatch):
    client = build_client()

    def handler(path):
        raise ApiHttpError(404, "ONEPOST API vrátilo HTTP_ERROR (HTTP 404)")

    install_request(monkeypatch, client, handler)

    try:
        client.get_attachments("does-not-exist")
    except ApiHttpError as e:
        assert e.status == 404
    else:
        raise AssertionError("expected ApiHttpError")


def test_permission_error_is_not_retried(monkeypatch):
    client = build_client()

    def handler(path):
        raise ApiPermissionError("ONEPOST nepovolil požadovanou akci (HTTP 403).")

    paths = install_request(monkeypatch, client, handler)

    try:
        client.get_attachments(ISDS_ID)
    except ApiPermissionError:
        pass
    else:
        raise AssertionError("expected ApiPermissionError")
    assert paths == [f"/messages/{ISDS_ID}/attachments"]


def test_server_error_is_not_retried(monkeypatch):
    client = build_client()

    def handler(path):
        raise ApiHttpError(500, "ONEPOST API je dočasně nedostupné (HTTP 500).")

    paths = install_request(monkeypatch, client, handler)

    try:
        client.get_attachments(ISDS_ID)
    except ApiHttpError as e:
        assert e.status == 500
    else:
        raise AssertionError("expected ApiHttpError")
    assert paths == [f"/messages/{ISDS_ID}/attachments"]


def test_non_object_message_response_keeps_the_original_error(monkeypatch):
    client = build_client()

    def handler(path):
        if path.endswith("/attachments"):
            raise ApiHttpError(404, "ONEPOST API vrátilo HTTP_ERROR (HTTP 404)")
        return []

    install_request(monkeypatch, client, handler)

    try:
        client.get_attachments(ISDS_ID)
    except ApiHttpError as e:
        assert e.status == 404
    else:
        raise AssertionError("expected ApiHttpError")


def test_message_without_a_usable_id_does_not_retry_itself(monkeypatch):
    client = build_client()

    def handler(path):
        if path.endswith("/attachments"):
            raise ApiHttpError(404, "ONEPOST API vrátilo HTTP_ERROR (HTTP 404)")
        return {"id": ISDS_ID, "isdsMessageId": ISDS_ID}

    paths = install_request(monkeypatch, client, handler)

    try:
        client.get_attachments(ISDS_ID)
    except ApiHttpError as e:
        assert e.status == 404
    else:
        raise AssertionError("expected ApiHttpError")
    assert paths == [f"/messages/{ISDS_ID}/attachments", f"/messages/{ISDS_ID}"]
