"""Store this plugin's settings through `cdxctl appenv`.

The plugin runs inside the isolation, where `~/.cdx/apps/onepost/.env` is
read-only; `cdxctl` asks the daemon to rewrite the file from outside. Reading
stays in `settings`, which parses the same file as before.
"""

import os
import shutil
import subprocess

from .exceptions import OnepostError


class AppEnvError(OnepostError):
    """`cdxctl` exited nonzero or could not be started."""

    def __init__(self, message: str, exit_code: int | None = None):
        super().__init__(message)
        self.message = message
        self.exit_code = exit_code


def set_value(app: str, name: str, value: str) -> None:
    """`cdxctl appenv <app> set <name> <value>`."""
    _run(["appenv", app, "set", name, value])


def delete_value(app: str, name: str) -> None:
    """`cdxctl appenv <app> delete <name>`."""
    _run(["appenv", app, "delete", name])


def _cdxctl_program() -> str:
    """`cdxctl` from PATH, else the location the codexis plugin installs it in."""
    found = shutil.which("cdxctl")
    if found:
        return found
    for home in (os.environ.get("HOME"), os.environ.get("CODEXIS_PUBLIC_USER_HOME")):
        if not home:
            continue
        candidate = os.path.join(home, ".local", "bin", "cdxctl")
        if os.path.isfile(candidate):
            return candidate
    return "cdxctl"


def _run(args: list[str]) -> None:
    try:
        result = subprocess.run(
            [_cdxctl_program(), *args],
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as e:
        raise AppEnvError(f"cdxctl se nepodařilo spustit: {e}") from e
    if result.returncode == 0:
        return
    detail = (result.stderr or "").strip()
    raise AppEnvError(
        f"cdxctl appenv skončil s kódem {result.returncode}: {detail}",
        exit_code=result.returncode,
    )
