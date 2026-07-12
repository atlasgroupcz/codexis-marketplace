"""Shared daemon (VM) auth + URL helpers for the plugin's daemon clients.

The daemon plugin API (e-mail, in-app notification, LLM extract) is authed with
the VM/daemon token the host writes to ~/.cdx/env/secret — the same token cdxctl
reads. NOT CODEXIS_USER_API_TOKEN: that JWT is for external Codexis sources and
the gateway rejects it (403).
"""

import os

_SECRET_FILE = os.path.join(".cdx", "env", "secret")


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
        token = token[len("Authorization:"):].strip()
    if token[:7].lower() == "bearer ":
        return token
    return f"Bearer {token}"


def rest_base(daemon_url):
    """REST base from the daemon URL, which is the GraphQL URL (…/graphql)."""
    base = (daemon_url or "").rstrip("/")
    if base.endswith("/graphql"):
        base = base[: -len("/graphql")].rstrip("/")
    return base
