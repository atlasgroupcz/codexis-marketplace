"""API key management for ČÚZK access.

The key is stored in ~/.cdx/apps/katastr/.env in a simple KEY=VALUE format
shared with the legacy `kn` tool and the HTTP backend.
"""

import os
import urllib.error
import urllib.request

from .exceptions import ApiKeyInvalidError, ApiHttpError, ApiNetworkError
from .fs import atomic_replace

_USER_HOME = os.environ.get("CDX_USER_HOME") or os.path.expanduser("~")
ENV_DIR = os.path.join(_USER_HOME, ".cdx", "apps", "katastr")
ENV_FILE = os.path.join(ENV_DIR, ".env")
CUZK_BASE = "https://api-kn.cuzk.gov.cz"
HEALTH_CHECK_PATH = "/api/v1/AplikacniSluzby/StavUctu"


def read_api_key() -> str:
    """Return the stored API key, or empty string if not configured."""
    if not os.path.isfile(ENV_FILE):
        return ""
    try:
        with open(ENV_FILE, encoding="utf-8") as f:
            for line in f:
                if line.startswith("API_KEY="):
                    return line.strip().split("=", 1)[1]
    except OSError:
        pass
    return ""


def write_api_key(key: str) -> None:
    """Write API key to disk atomically with mode 0600 (no validation)."""
    atomic_replace(ENV_FILE, lambda f: f.write(f"API_KEY={key}\n"), mode=0o600)


def mask_key(key: str) -> str:
    """Mask key for display: ABCD****WXYZ. Always masks at least 8 chars."""
    if len(key) < 12:
        return "****"
    return f"{key[:4]}****{key[-4:]}"


def is_configured() -> bool:
    """True if an API key is stored."""
    return bool(read_api_key())


def validate_key(key: str) -> None:
    """Validate API key against ČÚZK. Raises on failure.

    Raises:
        ApiKeyInvalidError: when ČÚZK returns 401/403
        ApiHttpError: for other HTTP errors
        ApiNetworkError: for connection issues
    """
    req = urllib.request.Request(
        CUZK_BASE + HEALTH_CHECK_PATH,
        headers={"ApiKey": key, "Accept": "application/json"},
    )
    try:
        urllib.request.urlopen(req, timeout=10)
    except urllib.error.HTTPError as e:
        if e.code in (401, 403):
            raise ApiKeyInvalidError(f"API klíč je neplatný (HTTP {e.code}).")
        raise ApiHttpError(e.code, f"ČÚZK API vrátilo HTTP {e.code}")
    except Exception as e:
        raise ApiNetworkError(f"Nepodařilo se kontaktovat ČÚZK API: {e}")


def set_api_key(key: str) -> None:
    """Validate and store API key."""
    key = (key or "").strip()
    if not key:
        raise ApiKeyInvalidError("API klíč je prázdný.")
    validate_key(key)
    write_api_key(key)
