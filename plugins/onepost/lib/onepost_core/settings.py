"""Settings for ONEPOST API access."""

import os
from urllib.parse import urlparse

from .exceptions import ConfigError
from .fs import atomic_replace

USER_HOME = os.environ.get("CODEXIS_PUBLIC_USER_HOME") or os.path.expanduser("~")
ENV_DIR = os.path.join(USER_HOME, ".cdx", "apps", "onepost")
ENV_FILE = os.path.join(ENV_DIR, ".env")
DEFAULT_ONEPOST_URL = "https://app.onepost.cz"
DEFAULT_API_URL = DEFAULT_ONEPOST_URL
DEFAULT_API_KEY_URL = f"{DEFAULT_ONEPOST_URL}/profil/codexis-ai-agent"
DEFAULT_DAEMON_ORIGIN = "https://vm.dev.internal.codexis.ai"


def _read_env_file() -> dict[str, str]:
    values: dict[str, str] = {}
    if not os.path.isfile(ENV_FILE):
        return values
    try:
        with open(ENV_FILE, encoding="utf-8") as f:
            for line in f:
                stripped = line.strip()
                if not stripped or stripped.startswith("#") or "=" not in stripped:
                    continue
                key, value = stripped.split("=", 1)
                values[key.strip()] = value.strip()
    except OSError:
        return {}
    return values


def _write_env_file(values: dict[str, str]) -> None:
    def writer(f):
        for key in sorted(values):
            if values[key]:
                f.write(f"{key}={values[key]}\n")

    atomic_replace(ENV_FILE, writer, mode=0o600)


def normalize_api_url(url: str) -> str:
    cleaned = (url or "").strip().rstrip("/")
    if not cleaned:
        raise ConfigError("ONEPOST API URL je prázdná.")
    parsed = urlparse(cleaned)
    if parsed.scheme not in ("http", "https") or not parsed.netloc:
        raise ConfigError("ONEPOST API URL musí být absolutní HTTP(S) URL.")
    return cleaned


def normalize_origin_url(url: str) -> str:
    cleaned = (url or "").strip().rstrip("/")
    if not cleaned:
        raise ConfigError("ONEPOST API origin je prázdný.")
    parsed = urlparse(cleaned)
    if parsed.scheme not in ("http", "https") or not parsed.netloc:
        raise ConfigError("ONEPOST API origin musí být absolutní HTTP(S) URL.")
    return f"{parsed.scheme}://{parsed.netloc}"


def read_api_url() -> str:
    value = (
        os.environ.get("ONEPOST_API_URL")
        or _read_env_file().get("ONEPOST_API_URL")
        or os.environ.get("CODEXIS_PLUGIN_ONEPOST_API_URL")
        or DEFAULT_API_URL
    )
    return normalize_api_url(value)


def read_api_origin() -> str:
    value = (
        os.environ.get("ONEPOST_API_ORIGIN")
        or _read_env_file().get("ONEPOST_API_ORIGIN")
        or os.environ.get("CODEXIS_PLUGIN_ONEPOST_API_ORIGIN")
        or DEFAULT_DAEMON_ORIGIN
    )
    return normalize_origin_url(value)


def read_api_key_url() -> str:
    value = (
        os.environ.get("ONEPOST_API_KEY_URL")
        or _read_env_file().get("ONEPOST_API_KEY_URL")
        or os.environ.get("CODEXIS_PLUGIN_ONEPOST_API_KEY_URL")
        or DEFAULT_API_KEY_URL
    )
    return normalize_api_url(value)


def read_api_key() -> str:
    return (
        os.environ.get("CODEXIS_USER_ONEPOST_API_KEY")
        or os.environ.get("ONEPOST_API_KEY")
        or _read_env_file().get("ONEPOST_API_KEY")
        or _read_env_file().get("API_KEY")
        or ""
    ).strip()


def read_api_key_source() -> str:
    if os.environ.get("CODEXIS_USER_ONEPOST_API_KEY"):
        return "CODEXIS_USER_ONEPOST_API_KEY"
    if os.environ.get("ONEPOST_API_KEY"):
        return "ONEPOST_API_KEY"
    values = _read_env_file()
    if values.get("ONEPOST_API_KEY"):
        return ENV_FILE
    if values.get("API_KEY"):
        return ENV_FILE
    return ""


def write_api_key(key: str) -> None:
    values = _read_env_file()
    values["ONEPOST_API_KEY"] = key.strip()
    values.pop("API_KEY", None)
    _write_env_file(values)


def write_api_url(url: str) -> None:
    values = _read_env_file()
    values["ONEPOST_API_URL"] = normalize_api_url(url)
    _write_env_file(values)


def mask_key(key: str) -> str:
    if len(key) < 12:
        return "****"
    return f"{key[:4]}****{key[-4:]}"


def is_configured() -> bool:
    return bool(read_api_key())
