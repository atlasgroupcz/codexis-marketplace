from pathlib import Path

from tests.harness.output import Results
from tests.tiers.marketplace import test_plugin as run_plugin_test


class LockedPluginClient:
    def __init__(self, installation_error: RuntimeError | None):
        self.installation_error = installation_error
        self.install_calls: list[str] = []

    def install_plugin(self, plugin_id: str) -> dict:
        self.install_calls.append(plugin_id)
        if self.installation_error:
            raise self.installation_error
        return {"id": plugin_id, "name": "licensed-plugin"}


def locked_plugin() -> dict:
    return {
        "id": "plugin-id",
        "name": "licensed-plugin",
        "skills": [],
        "access": {"locked": True, "requiredPart": "MONITOR_DANE_AI"},
    }


def test_locked_plugin_passes_when_installation_is_rejected():
    client = LockedPluginClient(RuntimeError("licence required"))
    results = Results(record=True)

    run_plugin_test(client, locked_plugin(), Path("."), results)

    assert client.install_calls == ["plugin-id"]
    assert (results.passed, results.failed) == (1, 0)


def test_locked_plugin_fails_when_installation_succeeds():
    client = LockedPluginClient(None)
    results = Results(record=True)

    run_plugin_test(client, locked_plugin(), Path("."), results)

    assert client.install_calls == ["plugin-id"]
    assert (results.passed, results.failed) == (0, 1)
    assert results.errors == [
        "Locked plugin was installed without its required licence"
    ]
