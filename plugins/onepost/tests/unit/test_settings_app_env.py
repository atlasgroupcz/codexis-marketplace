"""Writing a setting goes to `cdxctl appenv`, never to the credential file.

The file is read-only inside the plugin's isolation, so what these assert is the
command line the plugin asks the platform with. A fake `cdxctl` on `PATH`
records its argv and can be told to fail.
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
def settings(tmp_path, monkeypatch):
    """`onepost_core.settings` with a scratch home and a fake `cdxctl` on PATH."""
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
    monkeypatch.delenv("ONEPOST_API_KEY", raising=False)
    monkeypatch.delenv("CODEXIS_USER_ONEPOST_API_KEY", raising=False)

    import onepost_core.settings as module

    module = importlib.reload(module)
    module.fake = FakeCdxctl(str(log), str(exit_file))
    return module


def test_saving_a_key_sets_it_and_clears_the_legacy_name(settings):
    settings.write_api_key("  tok-123456789  ")

    assert settings.fake.calls == [
        "appenv onepost set ONEPOST_API_KEY tok-123456789",
        "appenv onepost delete API_KEY",
    ]
    assert not os.path.exists(settings.ENV_FILE), "the plugin wrote the credential file"


def test_saving_an_empty_key_deletes_both_names(settings):
    settings.write_api_key("")

    assert settings.fake.calls == [
        "appenv onepost delete ONEPOST_API_KEY",
        "appenv onepost delete API_KEY",
    ]


def test_saving_the_url_normalizes_it_before_storing(settings):
    settings.write_api_url("https://app.onepost.cz/")

    assert settings.fake.calls == [
        "appenv onepost set ONEPOST_API_URL https://app.onepost.cz"
    ]


def test_a_failing_cdxctl_raises_with_the_exit_code_and_a_czech_message(settings):
    settings.fake.fail_with(7)

    with pytest.raises(settings.AppEnvError) as raised:
        settings.write_api_key("tok-123456789")

    assert raised.value.exit_code == 7
    assert "Uložení API klíče selhalo" in str(raised.value)
    assert "fake failure" in str(raised.value)


def test_reading_still_comes_from_the_credential_file(settings):
    os.makedirs(settings.ENV_DIR, exist_ok=True)
    with open(settings.ENV_FILE, "w", encoding="utf-8") as f:
        f.write("ONEPOST_API_KEY=from-the-file\n")

    assert settings.read_api_key() == "from-the-file"
    assert settings.read_api_key_source() == settings.ENV_FILE
