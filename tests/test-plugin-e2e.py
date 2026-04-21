"""Plugin E2E test suite.

Discovers plugins/<name>/acceptance/e2e/*.yaml files and runs each as a
multi-turn chat test against a live cdx-daemon.

Invocation (CI):
  pytest tests/test-plugin-e2e.py \
    --daemon URL --token JWT \
    --git-url URL --git-ref REF --base-ref REF \
    --changed

Invocation (local full run):
  pytest tests/test-plugin-e2e.py \
    --daemon URL --token JWT --git-url URL --git-ref REF
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest
import yaml

# Make sibling modules importable (--import-mode=importlib means no packages).
sys.path.insert(0, str(Path(__file__).resolve().parent))

from _assertions import AssertionFailure, apply_captures, run_step_assertions, substitute
from _chat_runner import ChatRunner
from _changed_plugins import get_changed_plugins
from _daemon_client import DaemonClient, encode_node_id
from _transcript import write_transcript


# -------------------------------------------------------------- Collection
# NOTE: pytest_addoption is in tests/conftest.py — it must live there so that
# options are registered at argument-parse time, before collection begins.

MARKETPLACE_ROOT = Path(__file__).resolve().parent.parent


def _plugins_to_test(config) -> list[str]:
    only = (config.getoption("--only") or "").strip()
    if only:
        return sorted({n.strip() for n in only.split(",") if n.strip()})
    if config.getoption("--changed"):
        base = config.getoption("--base-ref") or "origin/main"
        return get_changed_plugins(base, MARKETPLACE_ROOT)
    # Default: every plugin that has acceptance/e2e/
    return sorted(
        p.name for p in (MARKETPLACE_ROOT / "plugins").iterdir()
        if p.is_dir() and (p / "acceptance" / "e2e").is_dir()
    )


def _yaml_files_for(plugin: str) -> list[Path]:
    d = MARKETPLACE_ROOT / "plugins" / plugin / "acceptance" / "e2e"
    if not d.is_dir():
        return []
    return sorted(d.glob("*.yaml"))


def pytest_generate_tests(metafunc):
    if "yaml_path" not in metafunc.fixturenames:
        return
    cases: list[tuple[str, Path]] = []
    for plugin in _plugins_to_test(metafunc.config):
        for yml in _yaml_files_for(plugin):
            cases.append((plugin, yml))
    # Always parametrize (even with empty list) so pytest knows the fixture
    # names are handled. When cases is empty pytest will report "no tests
    # collected" (exit code 5) after the NOTSET item is deselected by the
    # empty argvalues — this is the expected zero-YAML behaviour.
    metafunc.parametrize(
        ("plugin_name", "yaml_path"), cases,
        ids=[f"{p}/{y.stem}" for p, y in cases],
    )


# -------------------------------------------------------------- Fixtures

@pytest.fixture(scope="session")
def daemon_client(request) -> DaemonClient:
    url = request.config.getoption("--daemon") or os.environ.get("DAEMON_URL")
    tok = request.config.getoption("--token") or os.environ.get("SERVICE_JWT")
    if not url or not tok:
        pytest.skip("--daemon and --token required")
    c = DaemonClient(url, tok)
    if not c.health_check():
        pytest.skip("daemon not healthy")
    return c


@pytest.fixture(scope="session")
def marketplace(request, daemon_client) -> dict:
    """Add the marketplace from the given git ref, tear down at session end."""
    git_url = request.config.getoption("--git-url")
    git_ref = request.config.getoption("--git-ref")
    if not git_url or not git_ref:
        pytest.skip("--git-url and --git-ref required")

    from _daemon_client import ADD_MARKETPLACE, LIST_MARKETPLACES, REMOVE_MARKETPLACE
    manifest = yaml.safe_load((MARKETPLACE_ROOT / ".claude-plugin" / "marketplace.json").read_text())
    mkt_name = manifest["name"]
    mkt_uuid = manifest.get("uuid", "")

    # Remove any stale copy from a previous run.
    try:
        daemon_client.gql_data(REMOVE_MARKETPLACE,
                               {"id": encode_node_id("Marketplace", mkt_uuid)})
    except Exception:
        pass

    data = daemon_client.gql_data(
        ADD_MARKETPLACE,
        {"input": {"sourceType": "GIT", "gitUrl": git_url, "gitRef": git_ref}},
    )
    our = next((m for m in data["addMarketplace"] if m["name"] == mkt_name), None)
    assert our is not None, f"Marketplace {mkt_name!r} not found in addMarketplace reply"

    yield our

    try:
        daemon_client.gql_data(REMOVE_MARKETPLACE, {"id": our["id"]})
    except Exception:
        pass


@pytest.fixture(scope="module")
def _installed_plugins_cache():
    # Cross-test cache so we install each plugin exactly once per session.
    return {"installed": set()}


@pytest.fixture
def installed_plugin(request, daemon_client, marketplace, plugin_name,
                     _installed_plugins_cache):
    """Install `plugin_name` if not already, uninstall at session end."""
    from _daemon_client import INSTALL_PLUGIN, UNINSTALL_PLUGIN

    plugin = next((p for p in marketplace["plugins"] if p["name"] == plugin_name), None)
    if plugin is None:
        pytest.fail(f"plugin {plugin_name!r} not found in marketplace")

    if plugin["id"] not in _installed_plugins_cache["installed"]:
        daemon_client.gql_data(INSTALL_PLUGIN, {"input": {"id": plugin["id"]}})
        _installed_plugins_cache["installed"].add(plugin["id"])

        # Register session-end uninstall via finalizer.
        def _uninstall():
            try:
                daemon_client.gql_data(UNINSTALL_PLUGIN, {"input": {"id": plugin["id"]}})
            except Exception:
                pass
        request.session.addfinalizer(_uninstall)

    return plugin


# -------------------------------------------------------------- Test

def test_plugin_e2e(request, daemon_client, installed_plugin,
                    plugin_name, yaml_path):
    spec = yaml.safe_load(yaml_path.read_text())
    if spec.get("skip"):
        pytest.skip(f"skip: {spec['skip']}")

    test_name = spec.get("name") or yaml_path.stem
    steps = spec.get("steps") or []
    assert steps, f"{yaml_path}: no steps defined"

    runner = ChatRunner(
        daemon_client,
        poll_interval_s=float(os.environ.get("CDX_E2E_POLL_INTERVAL_S", "2")),
        poll_timeout_s=float(os.environ.get("CDX_E2E_POLL_TIMEOUT_S", "600")),
    )
    runner.start()

    import secrets
    captured: dict = {"run_id": secrets.token_hex(2)}
    recorded: list[dict] = []

    try:
        for step in steps:
            prompt = substitute(step["prompt"], captured)
            result = runner.step(prompt)
            expect = step.get("expect") or {}
            try:
                run_step_assertions(expect, result, captured)
                captured.update(apply_captures(expect.get("capture"), result))
                recorded.append({"prompt": prompt, "result": result, "status": "PASS"})
            except (AssertionFailure, AssertionError) as e:
                recorded.append({"prompt": prompt, "result": result,
                                 "status": "FAIL", "error": str(e)})
                raise
    except BaseException:
        out_dir = Path(request.config.getoption("--transcript-dir"))
        if not out_dir.is_absolute():
            out_dir = MARKETPLACE_ROOT / out_dir
        write_transcript(out_dir, plugin_name, test_name, recorded)
        raise
