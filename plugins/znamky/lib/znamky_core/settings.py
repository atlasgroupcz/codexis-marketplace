"""EUIPO API credential management (user-provided).

The official EUIPO trademark-search API is OAuth2-protected, so the user brings
their own free credentials from dev.euipo.europa.eu. They live in
$CODEXIS_PUBLIC_USER_HOME/.cdx/apps/znamky/.env (KEY=VALUE, mode 0600). That file
is read-only inside the plugin's isolation, so `cdxctl appenv` (see
:mod:`app_env`) asks the daemon to rewrite it; this module only reads it.
"""

import base64
import os
import urllib.error
import urllib.request

from . import app_env
from .app_env import AppEnvError
from .exceptions import ApiHttpError, ApiNetworkError, CredentialsInvalidError

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


APP_ID = "znamky"
CLIENT_ID_VAR = "EUIPO_CLIENT_ID"
CLIENT_SECRET_VAR = "EUIPO_CLIENT_SECRET"


def read_credentials() -> tuple:
    """Return (client_id, client_secret); empty strings if not configured."""
    env = _read_env()
    return env.get(CLIENT_ID_VAR, ""), env.get(CLIENT_SECRET_VAR, "")


def write_credentials(client_id: str, client_secret: str) -> None:
    """Store both credentials through the platform, or remove both when empty.

    One `cdxctl` call per name, in a fixed order, so a failure halfway leaves the
    id written and the secret not - `is_configured` needs both, so that state
    still reads as unconfigured.
    """
    try:
        if client_id and client_secret:
            app_env.set_value(APP_ID, CLIENT_ID_VAR, client_id)
            app_env.set_value(APP_ID, CLIENT_SECRET_VAR, client_secret)
        else:
            app_env.delete_value(APP_ID, CLIENT_ID_VAR)
            app_env.delete_value(APP_ID, CLIENT_SECRET_VAR)
    except AppEnvError as e:
        raise AppEnvError(f"Uložení EUIPO klíčů selhalo: {e}", e.exit_code) from e


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
            raise CredentialsInvalidError(
                f"EUIPO credentials odmítnuty (HTTP {exc.code})."
            )
        raise ApiHttpError(exc.code, f"EUIPO auth vrátilo HTTP {exc.code}")
    except (urllib.error.URLError, OSError) as exc:
        raise ApiNetworkError(
            f"Nepodařilo se kontaktovat EUIPO auth: {getattr(exc, 'reason', exc)}"
        )


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
