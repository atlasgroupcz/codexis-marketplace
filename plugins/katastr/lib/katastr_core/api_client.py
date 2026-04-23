"""Thin HTTP client for ČÚZK REST API KN."""

import json
import urllib.error
import urllib.request

from .exceptions import (
    ApiHttpError,
    ApiKeyInvalidError,
    ApiKeyMissingError,
    ApiNetworkError,
)
from .settings import CUZK_BASE, read_api_key


def get(path: str) -> dict:
    """Authenticated GET against ČÚZK API. Returns parsed JSON.

    Raises:
        ApiKeyMissingError: when no API key is configured
        ApiKeyInvalidError: on HTTP 401/403
        ApiHttpError: on other HTTP errors
        ApiNetworkError: on connection issues
    """
    api_key = read_api_key()
    if not api_key:
        raise ApiKeyMissingError(
            "API klíč pro Katastr není nastaven. "
            "Nastavte ho v UI (Doplňky → Katastr → ⚙) nebo přes "
            "`katastr-cli settings set <KEY>`."
        )

    url = CUZK_BASE + path if path.startswith("/") else CUZK_BASE + "/" + path
    req = urllib.request.Request(
        url, headers={"ApiKey": api_key, "Accept": "application/json"}
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        if e.code in (401, 403):
            raise ApiKeyInvalidError(
                f"API klíč pro Katastr byl odmítnut (HTTP {e.code})."
            )
        body = e.read().decode("utf-8", errors="replace").strip()
        detail = f": {body}" if body else ""
        raise ApiHttpError(
            e.code, f"ČÚZK API vrátilo HTTP {e.code} pro {path}{detail}"
        )
    except urllib.error.URLError as e:
        raise ApiNetworkError(f"Nepodařilo se kontaktovat ČÚZK API: {e.reason}")
