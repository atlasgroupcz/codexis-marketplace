"""HTTP client boundary for ARES public APIs."""

from __future__ import annotations

import json
import os
import socket
import ssl
from pathlib import Path
from typing import Any
from urllib import error, request

from .errors import AresCliError


BASE_URL = "https://ares.gov.cz/ekonomicke-subjekty-v-be/rest"


class AresClient:
    """Small transport wrapper."""

    def __init__(self, *, base_url: str = BASE_URL, timeout: float = 20.0) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self.ssl_context = _ssl_context()

    def get_json(self, path: str) -> dict[str, Any]:
        return self._request_json("GET", path)

    def post_json(self, path: str, payload: dict[str, Any]) -> dict[str, Any]:
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        return self._request_json("POST", path, body=body)

    def _request_json(self, method: str, path: str, body: bytes | None = None) -> dict[str, Any]:
        url = f"{self.base_url}{path}"
        headers = {
            "Accept": "application/json",
            "User-Agent": "ares-cli/0.1",
        }
        if body is not None:
            headers["Content-Type"] = "application/json; charset=utf-8"

        req = request.Request(url, data=body, headers=headers, method=method)
        try:
            with request.urlopen(req, timeout=self.timeout, context=self.ssl_context) as response:
                data = response.read()
        except error.HTTPError as exc:
            payload = exc.read()
            raise AresCliError(_http_error_message(method, path, exc.code, payload)) from exc
        except error.URLError as exc:
            raise AresCliError(f"{method} {path}: network error: {exc.reason}") from exc
        except socket.timeout as exc:
            raise AresCliError(f"{method} {path}: request timed out after {self.timeout:g}s") from exc

        if not data:
            return {}

        try:
            parsed = json.loads(data.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise AresCliError(f"{method} {path}: response is not valid JSON") from exc

        if not isinstance(parsed, dict):
            raise AresCliError(f"{method} {path}: expected JSON object, got {type(parsed).__name__}")
        return parsed


def _http_error_message(method: str, path: str, status: int, payload: bytes) -> str:
    detail = _decode_error_payload(payload)
    if detail:
        return f"{method} {path}: HTTP {status}: {detail}"
    return f"{method} {path}: HTTP {status}"


def _decode_error_payload(payload: bytes) -> str:
    if not payload:
        return ""
    try:
        parsed = json.loads(payload.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError):
        return payload.decode("utf-8", errors="replace").strip()

    if isinstance(parsed, dict):
        if isinstance(parsed.get("chyby"), list):
            return "; ".join(_format_chyba(item) for item in parsed["chyby"] if isinstance(item, dict))
        return _format_chyba(parsed)
    return str(parsed)


def _format_chyba(payload: dict[str, Any]) -> str:
    parts = [
        str(payload.get("kod") or "").strip(),
        str(payload.get("subKod") or "").strip(),
        str(payload.get("popis") or "").strip(),
    ]
    return " - ".join(part for part in parts if part)


def _ssl_context() -> ssl.SSLContext | None:
    if os.environ.get("SSL_CERT_FILE") or os.environ.get("SSL_CERT_DIR"):
        return None

    for candidate in _ca_candidates():
        if candidate.is_file():
            return ssl.create_default_context(cafile=str(candidate))
    return None


def _ca_candidates() -> list[Path]:
    candidates = [
        Path("/etc/ssl/cert.pem"),
        Path("/opt/homebrew/etc/openssl@3/cert.pem"),
        Path("/usr/local/etc/openssl@3/cert.pem"),
    ]
    try:
        import certifi  # type: ignore[import-not-found]
    except ImportError:
        return candidates
    return candidates + [Path(certifi.where())]
