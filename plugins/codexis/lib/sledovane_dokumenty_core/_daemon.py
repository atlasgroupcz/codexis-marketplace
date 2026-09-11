"""Shared daemon (VM) auth + URL helpers for the plugin's daemon clients.

The daemon plugin API (e-mail, in-app notification, LLM extract, GraphQL) is
authed with the VM/daemon token the host writes to ~/.cdx/env/secret - the same
token cdxctl reads. NOT CODEXIS_USER_API_TOKEN: that JWT is for external Codexis
sources and the gateway rejects it (403).
"""

import json
import os
import urllib.error
import urllib.request

_SECRET_FILE = os.path.join(".cdx", "env", "secret")
_GRAPHQL_PATH = "/graphql"


def load_auth():
    """Bearer authorization header from ~/.cdx/env/secret, or None if unavailable."""
    path = os.path.join(os.path.expanduser("~"), _SECRET_FILE)
    try:
        with open(path, encoding="utf-8") as f:
            token = f.read().strip()
    except OSError:
        return None
    if not token:
        return None
    if token.lower().startswith("authorization:"):
        token = token[len("Authorization:") :].strip()
    if token[:7].lower() == "bearer ":
        return token
    return f"Bearer {token}"


def daemon_url():
    return os.environ.get("CODEXIS_PUBLIC_DAEMON_URL")


def rest_base(daemon_url):
    """REST base from the daemon URL, which is the GraphQL URL (…/graphql)."""
    base = (daemon_url or "").rstrip("/")
    if base.endswith("/graphql"):
        base = base[: -len("/graphql")].rstrip("/")
    return base


def graphql_url():
    base = (daemon_url() or "").rstrip("/")
    if base.endswith(_GRAPHQL_PATH):
        return base
    return base + _GRAPHQL_PATH


def graphql(query, variables=None, timeout=10):
    """POST a GraphQL document to the daemon and return its ``data`` object.

    Best-effort: missing config or any transport/parse error yields None, so
    callers can treat "no daemon" and "bad answer" alike.
    """
    url, auth = daemon_url(), load_auth()
    if not url or not auth:
        return None
    payload = {"query": query}
    if variables:
        payload["variables"] = variables
    try:
        req = urllib.request.Request(
            graphql_url(),
            data=json.dumps(payload).encode("utf-8"),
            headers={"Content-Type": "application/json", "Authorization": auth},
        )
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = json.load(resp)
    except (urllib.error.URLError, OSError, ValueError):
        return None
    return data.get("data") if isinstance(data, dict) else None
