"""Thin HTTP client for the ONEPOST external REST API."""

import json
import mimetypes
import urllib.error
import urllib.parse
import urllib.request
import uuid
from pathlib import Path
from typing import Any

from . import settings
from .exceptions import (
    ApiAuthError,
    ApiHttpError,
    ApiKeyMissingError,
    ApiNetworkError,
    ApiPermissionError,
    ApiRateLimitError,
    OnepostError,
)

API_PREFIX = "/rest/api/v1"
USER_AGENT = "onepost-cli/0.5.0"


class OnepostClient:
    def __init__(self, api_url: str | None = None, api_key: str | None = None):
        self.api_url = settings.normalize_api_url(api_url or settings.read_api_url())
        self.api_key = (api_key if api_key is not None else settings.read_api_key()).strip()
        if not self.api_key:
            raise ApiKeyMissingError(
                "API klíč pro ONEPOST není nastaven. Vygenerujte ho v ONEPOSTu na "
                f"{settings.read_api_key_url()} a nastavte ho přes "
                "`onepost-cli settings set <KEY>` nebo jako `CODEXIS_USER_ONEPOST_API_KEY`."
            )

    def get_mailboxes(self, include_archive: bool = False) -> Any:
        return self.request("GET", "/mailboxes", {"includeArchive": include_archive})

    def get_folders(self, mailbox_id: str, parent_folder_id: str | None = None) -> Any:
        return self.request("GET", f"/mailboxes/{mailbox_id}/folders", {"parentFolderId": parent_folder_id})

    def get_message(self, message_id: str, include_attachments: bool = True) -> Any:
        return self.request("GET", f"/messages/{message_id}", {"includeAttachments": include_attachments})

    def get_folder_messages(
        self,
        folder_id: str,
        offset: int = 0,
        limit: int = 20,
        order_dir: str = "desc",
    ) -> Any:
        return self.request(
            "GET",
            f"/folders/{folder_id}/messages",
            {"offset": offset, "limit": limit, "orderDir": order_dir},
        )

    def search_messages(
        self,
        query: str,
        mailbox_id: str | None = None,
        folder_id: str | None = None,
        offset: int = 0,
        limit: int = 20,
    ) -> Any:
        return self.request(
            "GET",
            "/messages/search",
            {
                "q": query,
                "mailboxId": mailbox_id,
                "folderId": folder_id,
                "offset": offset,
                "limit": limit,
            },
        )

    def get_attachments(self, message_id: str) -> Any:
        return self._with_resolved_message_id(
            message_id,
            lambda resolved: self.request("GET", f"/messages/{resolved}/attachments"),
        )

    def _with_resolved_message_id(self, message_id: str, call) -> Any:
        try:
            return call(message_id)
        except ApiHttpError as e:
            if e.status != 404:
                raise
            internal_id = self._internal_message_id(message_id)
            if not internal_id or internal_id == message_id:
                raise
            return call(internal_id)

    def _internal_message_id(self, message_id: str) -> str | None:
        try:
            message = self.get_message(message_id, include_attachments=False)
        except OnepostError:
            return None
        if not isinstance(message, dict):
            return None
        internal_id = message.get("id")
        return internal_id if isinstance(internal_id, str) and internal_id else None

    def receive_messages(self, mailbox_id: str, only_headers: bool = False) -> Any:
        return self.request("POST", f"/mailboxes/{mailbox_id}/receive", {"onlyHeaders": only_headers})

    def upload_attachment(
        self,
        path: str,
        file_name: str | None = None,
        mime_type: str | None = None,
    ) -> Any:
        source = Path(path).expanduser()
        if not source.is_file():
            raise ApiHttpError(0, f"Příloha neexistuje nebo není soubor: {source}")

        resolved_file_name = file_name or source.name
        resolved_mime_type = mime_type or mimetypes.guess_type(resolved_file_name)[0] or "application/octet-stream"
        boundary, body = self._multipart_file_body(
            "file",
            resolved_file_name,
            resolved_mime_type,
            source.read_bytes(),
        )
        headers = self._headers()
        headers["Content-Type"] = f"multipart/form-data; boundary={boundary}"
        req = urllib.request.Request(
            self._build_url("/attachments", None),
            data=body,
            method="POST",
            headers=headers,
        )
        return self._read_json_response(req, 120)

    def save_concept(self, payload: dict[str, Any]) -> Any:
        return self.request("POST", "/concepts", body=payload)

    def send_concept(self, concept_id: str) -> Any:
        return self.request("POST", f"/concepts/{concept_id}/send", timeout=300)

    def download_attachment(self, attachment_id: str, output: str) -> dict[str, Any]:
        url = self._build_url(f"/attachments/{attachment_id}", None)
        headers = self._headers()
        headers["Accept"] = "application/octet-stream, application/json"
        req = urllib.request.Request(url, headers=headers)
        try:
            with urllib.request.urlopen(req, timeout=60) as resp:
                data = resp.read()
        except urllib.error.HTTPError as e:
            self._raise_http_error(e)
        except urllib.error.URLError as e:
            raise ApiNetworkError(f"Nepodařilo se kontaktovat ONEPOST API: {e.reason}") from e

        target = Path(output).expanduser()
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        return {"path": str(target), "bytes": len(data)}

    def request(
        self,
        method: str,
        path: str,
        query: dict[str, Any] | None = None,
        body: Any | None = None,
        timeout: int = 45,
    ) -> Any:
        data = None
        headers = self._headers()
        if body is not None:
            data = json.dumps(body).encode("utf-8")
            headers["Content-Type"] = "application/json"
        req = urllib.request.Request(
            self._build_url(path, query),
            data=data,
            method=method.upper(),
            headers=headers,
        )
        return self._read_json_response(req, timeout)

    def _read_json_response(self, req: urllib.request.Request, timeout: int) -> Any:
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                payload = resp.read()
                if not payload:
                    return None
                return json.loads(payload.decode("utf-8"))
        except urllib.error.HTTPError as e:
            self._raise_http_error(e)
        except urllib.error.URLError as e:
            raise ApiNetworkError(f"Nepodařilo se kontaktovat ONEPOST API: {e.reason}") from e
        except json.JSONDecodeError as e:
            raise ApiHttpError(200, f"ONEPOST API vrátilo neplatné JSON: {e}") from e

    @staticmethod
    def _multipart_file_body(
        field_name: str,
        file_name: str,
        mime_type: str,
        content: bytes,
    ) -> tuple[str, bytes]:
        boundary = f"----onepost-{uuid.uuid4().hex}"
        safe_field_name = OnepostClient._quote_header_value(field_name)
        safe_file_name = OnepostClient._quote_header_value(file_name)
        head = (
            f"--{boundary}\r\n"
            f"Content-Disposition: form-data; name=\"{safe_field_name}\"; filename=\"{safe_file_name}\"\r\n"
            f"Content-Type: {mime_type}\r\n\r\n"
        ).encode("utf-8")
        tail = f"\r\n--{boundary}--\r\n".encode("utf-8")
        return boundary, b"".join([head, content, tail])

    @staticmethod
    def _quote_header_value(value: str) -> str:
        return value.replace("\\", "\\\\").replace('"', '\\"').replace("\r", "_").replace("\n", "_")

    def _headers(self) -> dict[str, str]:
        return {
            "Accept": "application/json",
            "Origin": settings.read_api_origin(),
            "User-Agent": USER_AGENT,
            "X-API-Key": self.api_key,
        }

    def _build_url(self, path: str, query: dict[str, Any] | None) -> str:
        normalized_path = self._normalize_path(path)
        params = self._normalize_query(query or {})
        suffix = f"?{urllib.parse.urlencode(params)}" if params else ""
        return f"{self.api_url}{normalized_path}{suffix}"

    @staticmethod
    def _normalize_path(path: str) -> str:
        cleaned = (path or "").strip()
        if not cleaned:
            raise ApiHttpError(0, "API cesta je prázdná.")
        if cleaned.startswith("http://") or cleaned.startswith("https://"):
            raise ApiHttpError(0, "Použijte jen relativní ONEPOST API cestu, ne absolutní URL.")
        if not cleaned.startswith("/"):
            cleaned = "/" + cleaned
        if cleaned.startswith(API_PREFIX + "/") or cleaned == API_PREFIX:
            return cleaned
        return API_PREFIX + cleaned

    @staticmethod
    def _normalize_query(query: dict[str, Any]) -> dict[str, str]:
        params: dict[str, str] = {}
        for key, value in query.items():
            if value is None:
                continue
            if isinstance(value, bool):
                params[key] = "true" if value else "false"
            else:
                params[key] = str(value)
        return params

    def _raise_http_error(self, error: urllib.error.HTTPError) -> None:
        body = error.read().decode("utf-8", errors="replace").strip()
        code, message = self._parse_error_body(body)
        if error.code == 401:
            raise ApiAuthError(
                "ONEPOST odmítl API klíč (HTTP 401). Zkontrolujte, že je klíč opsaný správně, "
                "nebyl smazán a patří k aktuálnímu účtu."
            )
        if error.code == 403:
            raise ApiPermissionError(
                "ONEPOST nepovolil požadovanou akci (HTTP 403). Pro čtení stačí READ klíč, "
                "pro akce jako příjem zpráv, vytvoření konceptu nebo odeslání konceptu je potřeba "
                "READ_WRITE klíč. Může jít také o chybějící oprávnění k dané schránce, složce, "
                "zprávě nebo konceptu."
            )
        if error.code == 429:
            raise ApiRateLimitError("ONEPOST odmítl požadavek kvůli rate limitu (HTTP 429). Zkuste to později.")
        if 500 <= error.code <= 599:
            raise ApiHttpError(error.code, f"ONEPOST API je dočasně nedostupné (HTTP {error.code}).", code)
        detail = f": {message}" if message else ""
        label = code or "HTTP_ERROR"
        raise ApiHttpError(error.code, f"ONEPOST API vrátilo {label} (HTTP {error.code}){detail}", code)

    @staticmethod
    def _parse_error_body(body: str) -> tuple[str | None, str | None]:
        if not body:
            return None, None
        try:
            parsed = json.loads(body)
        except json.JSONDecodeError:
            return None, body[:500]
        if isinstance(parsed, dict):
            code = parsed.get("code")
            message = parsed.get("message") or parsed.get("error")
            return str(code) if code else None, str(message) if message else None
        return None, body[:500]
