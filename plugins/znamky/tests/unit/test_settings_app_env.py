"""Storing EUIPO credentials goes to `cdxctl appenv`, never to the file.

The credential file is read-only inside the plugin's isolation, so what these
assert is the command line the plugin asks the platform with. A fake `cdxctl` on
`PATH` records its argv and can be told to fail.
"""

import importlib
import os
import stat

import pytest

FAKE_CDXCTL = """#!/bin/sh
printf '%s\\n' "$*" >> "{log}"
code=$(cat "{exit_file}" 2>/dev/null || echo 0)
[ "$code" -eq 0 ] || echo "fake failure" >&2
exit "$code"
"""


class FakeCdxctl:
    def __init__(self, log, exit_file):
        self._log = log
        self._exit_file = exit_file

    @property
    def calls(self) -> list[str]:
        if not os.path.isfile(self._log):
            return []
        with open(self._log, encoding="utf-8") as f:
            return f.read().splitlines()

    def fail_with(self, code: int) -> None:
        with open(self._exit_file, "w", encoding="utf-8") as f:
            f.write(str(code))


@pytest.fixture
def cdxctl(tmp_path, monkeypatch):
    """A fake `cdxctl` on PATH, and a scratch home for the credential file."""
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    log = bin_dir / "calls.log"
    exit_file = bin_dir / "exit-code"
    script = bin_dir / "cdxctl"
    script.write_text(
        FAKE_CDXCTL.format(log=log, exit_file=exit_file), encoding="utf-8"
    )
    script.chmod(script.stat().st_mode | stat.S_IXUSR)

    monkeypatch.setenv("PATH", f"{bin_dir}{os.pathsep}{os.environ['PATH']}")
    monkeypatch.setenv("CODEXIS_PUBLIC_USER_HOME", str(tmp_path))

    return FakeCdxctl(str(log), str(exit_file))


@pytest.fixture
def settings(cdxctl):
    """`znamky_core.settings`, re-imported so it picks up the scratch home."""
    import znamky_core.settings

    return importlib.reload(znamky_core.settings)


def test_storing_both_credentials_makes_one_call_each(settings, cdxctl):
    settings.write_credentials("cid-1", "secret-1")

    assert cdxctl.calls == [
        "appenv znamky set EUIPO_CLIENT_ID cid-1",
        "appenv znamky set EUIPO_CLIENT_SECRET secret-1",
    ]
    assert not os.path.exists(settings.ENV_FILE), "the plugin wrote the credential file"


def test_clearing_the_credentials_deletes_both_names(settings, cdxctl):
    settings.write_credentials("", "")

    assert cdxctl.calls == [
        "appenv znamky delete EUIPO_CLIENT_ID",
        "appenv znamky delete EUIPO_CLIENT_SECRET",
    ]


def test_a_failing_cdxctl_raises_with_the_exit_code_and_a_czech_message(
    settings, cdxctl
):
    cdxctl.fail_with(7)

    with pytest.raises(settings.AppEnvError) as raised:
        settings.write_credentials("cid-1", "secret-1")

    assert raised.value.exit_code == 7
    assert "Uložení EUIPO klíčů selhalo" in str(raised.value)
    assert "fake failure" in str(raised.value)


def test_set_credentials_stores_only_after_euipo_accepts_them(
    settings, cdxctl, monkeypatch
):
    monkeypatch.setattr(settings, "fetch_token", lambda cid, secret: "tok")

    result = settings.set_credentials(" cid-1 ", " secret-1 ")

    assert result == {"verified": True, "warning": None}
    assert cdxctl.calls == [
        "appenv znamky set EUIPO_CLIENT_ID cid-1",
        "appenv znamky set EUIPO_CLIENT_SECRET secret-1",
    ]


def test_rejected_credentials_are_not_stored(settings, cdxctl, monkeypatch):
    def reject(cid, secret):
        raise settings.CredentialsInvalidError(
            "EUIPO credentials odmítnuty (HTTP 401)."
        )

    monkeypatch.setattr(settings, "fetch_token", reject)

    with pytest.raises(settings.CredentialsInvalidError):
        settings.set_credentials("cid-1", "secret-1")

    assert cdxctl.calls == []


def test_unverifiable_credentials_are_stored_with_a_warning(
    settings, cdxctl, monkeypatch
):
    def unreachable(cid, secret):
        raise settings.ApiNetworkError("Nepodařilo se kontaktovat EUIPO auth: timeout")

    monkeypatch.setattr(settings, "fetch_token", unreachable)

    result = settings.set_credentials("cid-1", "secret-1")

    assert result["verified"] is False
    assert "ověření selhalo" in result["warning"]
    assert cdxctl.calls == [
        "appenv znamky set EUIPO_CLIENT_ID cid-1",
        "appenv znamky set EUIPO_CLIENT_SECRET secret-1",
    ]


def test_reading_still_comes_from_the_credential_file(settings, cdxctl):
    os.makedirs(settings.ENV_DIR, exist_ok=True)
    with open(settings.ENV_FILE, "w", encoding="utf-8") as f:
        f.write(
            "EUIPO_CLIENT_ID=from-the-file\nEUIPO_CLIENT_SECRET=also-from-the-file\n"
        )

    assert settings.read_credentials() == ("from-the-file", "also-from-the-file")
    assert settings.is_configured()
