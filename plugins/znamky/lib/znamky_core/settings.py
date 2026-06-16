"""EUIPO API credential management (user-provided).

The official EUIPO trademark-search API is OAuth2-protected, so the user brings
their own free credentials from dev.euipo.europa.eu. They are stored in
$CODEXIS_PUBLIC_USER_HOME/.cdx/apps/znamky/.env (KEY=VALUE, mode 0600), the same
self-contained pattern the katastr plugin uses for its ČÚZK key.
"""

import base64
import os
import urllib.error
import urllib.request

from .exceptions import ApiHttpError, ApiNetworkError, CredentialsInvalidError
from .fs import atomic_replace

_USER_HOME = os.environ.get("CODEXIS_PUBLIC_USER_HOME") or os.path.expanduser("~")
ENV_DIR = os.path.join(_USER_HOME, ".cdx", "apps", "znamky")
ENV_FILE = os.path.join(ENV_DIR, ".env")

AUTH_URL = (
    os.environ.get("CODEXIS_PLUGIN_ZNAMKY_EUIPO_AUTH_URL")
    or "https://auth.euipo.europa.eu/oidc/accessToken"
)


def _read_env() -> dict:
    out = {}
    if not os.path.isfile(ENV_FILE):
        return out
    try:
        with open(ENV_FILE, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line and "=" in line and not line.startswith("#"):
                    k, v = line.split("=", 1)
                    out[k.strip()] = v.strip()
    except OSError:
        pass
    return out


def read_credentials() -> tuple:
    """Return (client_id, client_secret); empty strings if not configured."""
    env = _read_env()
    return env.get("EUIPO_CLIENT_ID", ""), env.get("EUIPO_CLIENT_SECRET", "")


def write_credentials(client_id: str, client_secret: str) -> None:
    body = f"EUIPO_CLIENT_ID={client_id}\nEUIPO_CLIENT_SECRET={client_secret}\n"
    atomic_replace(ENV_FILE, lambda f: f.write(body), mode=0o600)


def is_configured() -> bool:
    cid, secret = read_credentials()
    return bool(cid and secret)


def mask(value: str) -> str:
    if len(value) < 8:
        return "****"
    return f"{value[:3]}****{value[-3:]}"


def fetch_token(client_id: str, client_secret: str) -> str:
    """OAuth2 client-credentials token from EUIPO. Raises on auth/HTTP failure."""
    basic = base64.b64encode(f"{client_id}:{client_secret}".encode()).decode()
    data = b"grant_type=client_credentials&scope=uid"
    req = urllib.request.Request(
        AUTH_URL,
        data=data,
        method="POST",
        headers={
            "Authorization": f"Basic {basic}",
            "Content-Type": "application/x-www-form-urlencoded",
            "Accept": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            import json
            payload = json.loads(resp.read())
        token = payload.get("access_token")
        if not token:
            raise CredentialsInvalidError("EUIPO nevrátilo access_token.")
        return token
    except urllib.error.HTTPError as exc:
        if exc.code in (400, 401, 403):
            raise CredentialsInvalidError(f"EUIPO credentials odmítnuty (HTTP {exc.code}).")
        raise ApiHttpError(exc.code, f"EUIPO auth vrátilo HTTP {exc.code}")
    except (urllib.error.URLError, OSError) as exc:
        raise ApiNetworkError(f"Nepodařilo se kontaktovat EUIPO auth: {getattr(exc, 'reason', exc)}")


def set_credentials(client_id: str, client_secret: str) -> dict:
    """Store EUIPO credentials. Validates against EUIPO when reachable.

    Returns {'verified': bool, 'warning': str|None}. Auth rejection (bad creds)
    raises CredentialsInvalidError and stores nothing; a network error stores the
    creds anyway (can't verify from a restricted network) with a warning.
    """
    client_id = (client_id or "").strip()
    client_secret = (client_secret or "").strip()
    if not client_id or not client_secret:
        raise CredentialsInvalidError("Vyplňte client_id i client_secret.")
    verified = False
    warning = None
    try:
        fetch_token(client_id, client_secret)
        verified = True
    except CredentialsInvalidError:
        raise
    except (ApiNetworkError, ApiHttpError) as exc:
        warning = f"Klíče uloženy, ale ověření selhalo: {exc}"
    write_credentials(client_id, client_secret)
    return {"verified": verified, "warning": warning}
