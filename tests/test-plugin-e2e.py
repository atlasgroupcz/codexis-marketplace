#!/usr/bin/env python3
"""
Codexis Plugin E2E Test

For each plugin, installs it into a running cdx-daemon, drives a multi-turn
chat via every plugins/<name>/acceptance/e2e/*.yaml file, and asserts on tool
calls and outputs. Failure writes a Markdown transcript to
test-results/transcripts/<plugin>/<test>.md for debugging.

By default, only runs for plugins changed between --base-ref and HEAD.
Pass --all to run every plugin with acceptance/e2e/, or --only NAME[,NAME]
to restrict to specific plugins.

Usage:
  python3 tests/test-plugin-e2e.py \\
    --daemon http://localhost:8086 \\
    --token eyJhbGci... \\
    --git-url https://gitlab.agrp.dev/profidata/codexis-marketplace.git \\
    --git-ref feat/my-branch

Requires: pyyaml, jsonpath-ng
  pip install pyyaml jsonpath-ng
"""

from __future__ import annotations

import argparse
import json
import secrets
import sys
from pathlib import Path

import yaml

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _assertions import (
    AssertionFailure,
    do_graphql_captures,
    run_step_checks,
    substitute,
)
from _chat_runner import ChatRunner
from _changed_plugins import get_changed_plugins
from _daemon_client import DaemonClient
from _output import Results
from _transcript import write_transcript


MARKETPLACE_ROOT = Path(__file__).resolve().parent.parent


# ---------------------------------------------------------------------------
# Plugin selection
# ---------------------------------------------------------------------------
def _all_plugins_with_e2e() -> list[str]:
    plugins_dir = MARKETPLACE_ROOT / "plugins"
    if not plugins_dir.is_dir():
        return []
    return sorted(
        p.name for p in plugins_dir.iterdir()
        if p.is_dir() and (p / "acceptance" / "e2e").is_dir()
    )


def _yaml_files_for(plugin: str, yaml_filter: str = "") -> list[Path]:
    d = MARKETPLACE_ROOT / "plugins" / plugin / "acceptance" / "e2e"
    if not d.is_dir():
        return []
    files = sorted(d.glob("*.yaml"))
    if yaml_filter:
        wanted = {n.strip() for n in yaml_filter.split(",") if n.strip()}
        files = [f for f in files if f.stem in wanted or any(w in f.stem for w in wanted)]
    return files


def pick_plugins(args: argparse.Namespace, r: Results) -> list[str]:
    if args.only:
        names = sorted({n.strip() for n in args.only.split(",") if n.strip()})
        r.log(f"--only: {', '.join(names) or '(empty)'}")
        return names
    if args.all:
        names = _all_plugins_with_e2e()
        r.log(f"--all: {len(names)} plugin(s) with acceptance/e2e/")
        return names
    names = get_changed_plugins(args.base_ref, MARKETPLACE_ROOT)
    r.log(f"--changed vs {args.base_ref}: {len(names)} plugin(s)")
    return names


# ---------------------------------------------------------------------------
# Daemon setup / teardown
# ---------------------------------------------------------------------------
def preflight(client: DaemonClient, r: Results) -> dict:
    r.section("Pre-flight checks")

    manifest_path = MARKETPLACE_ROOT / ".claude-plugin" / "marketplace.json"
    if not manifest_path.exists():
        print(f"ERROR: marketplace.json not found at {manifest_path}", file=sys.stderr)
        sys.exit(1)
    r.ok("Marketplace manifest found")

    if not client.health_check():
        print("ERROR: Daemon not reachable or not healthy", file=sys.stderr)
        sys.exit(1)
    r.ok("Daemon is healthy")

    manifest = json.loads(manifest_path.read_text())
    r.log(f"Marketplace: {manifest['name']} (uuid={manifest.get('uuid', 'N/A')})")
    return manifest


def add_marketplace(client: DaemonClient, git_url: str, git_ref: str,
                    manifest: dict, r: Results) -> dict:
    r.section("Add marketplace (GIT)")
    mkt_name = manifest["name"]
    r.log(f"Git URL: {git_url}  ref: {git_ref}")

    try:
        our = client.add_marketplace_idempotent(git_url, git_ref, manifest)
    except RuntimeError as e:
        r.fail(f"Failed to add marketplace: {e}")
        sys.exit(1)

    if not our or our.get("name") != mkt_name:
        r.fail(f"Marketplace {mkt_name!r} not in response: {our!r}")
        sys.exit(1)
    r.ok("Marketplace added")
    return our


def remove_marketplace(client: DaemonClient, mkt_node_id: str, r: Results) -> None:
    r.section("Remove marketplace")
    try:
        client.remove_marketplace(mkt_node_id)
        r.ok("Marketplace removed")
    except RuntimeError as e:
        r.fail(f"Failed to remove marketplace: {e}")


# ---------------------------------------------------------------------------
# Optional setup block: runner places fixtures/folders before chat starts.
# Supported kinds:
#   upload:
#     destination: <vm path>
#     files:
#       - path: <relative/path>
#         content: <string>             # inline text (templated with vars)
#         # OR:
#         source: <path-relative-to-yaml>   # read raw bytes from disk (binary)
# ---------------------------------------------------------------------------
def _run_setup(client: DaemonClient, setup: list[dict], captured: dict,
               yaml_dir: Path) -> None:
    from _assertions import substitute
    for i, action in enumerate(setup):
        if "upload" not in action:
            raise ValueError(f"setup[{i}]: unknown kind, expected 'upload', got {list(action)!r}")
        block = action["upload"]
        destination = substitute(block["destination"], captured)
        files: list[tuple[str, bytes | str]] = []
        for f in block.get("files") or []:
            rel = substitute(f["path"], captured)
            if "content" in f:
                content: bytes | str = substitute(f["content"], captured)
            elif "source" in f:
                src = yaml_dir / substitute(f["source"], captured)
                content = src.read_bytes()
            else:
                raise ValueError(
                    f"setup[{i}] file {rel!r}: needs either 'content' or 'source'"
                )
            files.append((rel, content))
        client.upload_folder(destination, files)


# ---------------------------------------------------------------------------
# One YAML = one multi-turn chat test
# ---------------------------------------------------------------------------
def run_yaml(client: DaemonClient, plugin_name: str, yaml_path: Path,
             transcript_dir: Path, args: argparse.Namespace, r: Results,
             builtin_vars: dict) -> None:
    spec = yaml.safe_load(yaml_path.read_text())
    test_name = spec.get("name") or yaml_path.stem

    if spec.get("skip"):
        r.skip(f"{plugin_name}/{test_name}: {spec['skip']}")
        return

    steps = spec.get("steps") or []
    if not steps:
        r.fail(f"{plugin_name}/{test_name}: no steps defined")
        return

    # All-letters run_id: some daemon validators (e.g. agent/skill names) only allow
    # lowercase letters and hyphens, so avoid digits.
    captured: dict = {
        "run_id": "".join(secrets.choice("abcdefghijklmnop") for _ in range(4)),
        **builtin_vars,
    }

    # Run optional `setup:` block BEFORE the chat starts. The runner (not the AI)
    # puts fixture files/folders into place so each step can focus on actually
    # testing the plugin rather than on scaffolding. See _run_setup() below.
    try:
        _run_setup(client, spec.get("setup") or [], captured, yaml_path.parent)
    except Exception as e:
        r.fail(f"{plugin_name}/{test_name}: setup failed: {e}")
        return

    runner = ChatRunner(
        client,
        poll_interval_s=args.poll_interval_s,
        poll_timeout_s=args.poll_timeout_s,
    )
    runner.start()

    recorded: list[dict] = []
    failure: str | None = None

    for i, step in enumerate(steps, start=1):
        prompt = substitute(step["prompt"], captured)
        try:
            result = runner.step(prompt)
        except Exception as e:
            failure = f"step {i} chat error: {e}"
            recorded.append({"prompt": prompt,
                             "result": {"text": "", "tool_calls": []},
                             "status": "FAIL", "error": failure})
            break

        expect = step.get("expect") or {}
        try:
            run_step_checks(expect, result, captured, client)
            # Step passed — run captures (if any) to make values available
            # to subsequent steps as {{ name }}.
            captured.update(do_graphql_captures(step.get("capture"), client, captured))
            recorded.append({"prompt": prompt, "result": result, "status": "PASS"})
        except (AssertionFailure, AssertionError) as e:
            failure = f"step {i}: {e}"
            recorded.append({"prompt": prompt, "result": result,
                             "status": "FAIL", "error": str(e)})
            break

    if failure:
        transcript_path = write_transcript(transcript_dir, plugin_name, test_name, recorded)
        r.fail(f"{plugin_name}/{test_name}: {failure}")
        r.log(f"  transcript: {transcript_path}")
    else:
        r.ok(f"{plugin_name}/{test_name}")


def test_plugin(client: DaemonClient, plugin: dict, args: argparse.Namespace,
                transcript_dir: Path, r: Results, builtin_vars: dict) -> None:
    name = plugin["name"]
    plugin_id = plugin["id"]
    yamls = _yaml_files_for(name, args.yaml)

    r.section(f"Plugin: {name}")
    if not yamls:
        r.skip(f"{name}: no acceptance/e2e/*.yaml files")
        return

    r.log("Installing…")
    # Uninstall first (ignore errors) to guarantee postInstall fires fresh.
    # Without this, a plugin left installed from a prior crashed run causes
    # the next install_plugin to no-op — leaving PATH / binaries in a stale
    # state and making every AI call pay for binary discovery overhead.
    try:
        client.uninstall_plugin(plugin_id)
    except RuntimeError:
        pass
    try:
        client.install_plugin(plugin_id)
    except RuntimeError as e:
        r.fail(f"{name}: install failed: {e}")
        return

    try:
        from concurrent.futures import ThreadPoolExecutor
        with ThreadPoolExecutor(max_workers=len(yamls)) as pool:
            futs = [pool.submit(run_yaml, client, name, y, transcript_dir, args, r, builtin_vars)
                    for y in yamls]
            for f in futs:
                f.result()
    finally:
        r.log("Uninstalling…")
        try:
            client.uninstall_plugin(plugin_id)
        except RuntimeError as e:
            r.fail(f"{name}: uninstall failed: {e}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main() -> int:
    parser = argparse.ArgumentParser(description="Codexis Plugin E2E Test")
    parser.add_argument("--daemon", required=True,
                        help="Daemon URL (e.g. http://localhost:8086)")
    parser.add_argument("--token", required=True, help="JWT token")
    parser.add_argument("--git-url", required=True, help="Git repository URL")
    parser.add_argument("--git-ref", required=True,
                        help="Git branch or tag (merge-result ref in CI)")
    parser.add_argument("--base-ref", default="origin/main",
                        help="Base ref for --changed diff (default: origin/main)")
    mx = parser.add_mutually_exclusive_group()
    mx.add_argument("--all", action="store_true",
                    help="Run every plugin that has acceptance/e2e/ (override default)")
    mx.add_argument("--only", default="",
                    help="Comma-separated plugin names (override default)")
    parser.add_argument("--yaml", default="",
                        help="Comma-separated YAML stem names/substrings to filter which "
                             "plugin test cases run (e.g. 'agent,skill'); applied after --only")
    parser.add_argument("--transcript-dir", default="test-results/transcripts",
                        help="Where to write failure transcripts")
    parser.add_argument("--poll-interval-s", type=float, default=2.0,
                        help="Chat polling interval seconds (default: 2)")
    parser.add_argument("--poll-timeout-s", type=float, default=600.0,
                        help="Chat polling timeout seconds (default: 600)")
    parser.add_argument("--cookie", default="",
                        help="Optional _oauth2_proxy cookie value used to "
                             "auto-refresh the bearer token on 401 (long suites "
                             "outlast the ~30min keycloak TTL otherwise).")
    parser.add_argument("--oauth2-proxy", default="http://localhost:4182",
                        help="oauth2-proxy URL (default: http://localhost:4182)")
    parser.add_argument("--var", action="append", default=[],
                        metavar="NAME=VALUE",
                        help="Inject a template variable available to YAML "
                             "prompts and setup blocks (e.g. --var "
                             "katastr_api_key=...). Repeatable.")
    args = parser.parse_args()

    transcript_dir = Path(args.transcript_dir)
    if not transcript_dir.is_absolute():
        transcript_dir = MARKETPLACE_ROOT / transcript_dir

    refresher = None
    if args.cookie:
        from _daemon_client import make_oauth2_proxy_refresher
        refresher = make_oauth2_proxy_refresher(args.cookie, args.oauth2_proxy)
    client = DaemonClient(args.daemon, args.token, token_refresher=refresher)
    r = Results()

    r.log(f"Daemon: {args.daemon}")
    r.log(f"Marketplace: {MARKETPLACE_ROOT}")

    manifest = preflight(client, r)
    plugin_names = pick_plugins(args, r)

    if not plugin_names:
        r.log("No plugins to test — done.")
        return r.summary()

    our_mkt = add_marketplace(client, args.git_url, args.git_ref, manifest, r)
    mkt_plugins_by_name = {p["name"]: p for p in our_mkt.get("plugins", [])}
    builtin_vars = {
        "marketplace_id": our_mkt["id"],
        "marketplace_name": our_mkt["name"],
    }
    # User-supplied --var NAME=VALUE entries become template vars too.
    for v in args.var:
        if "=" not in v:
            r.fail(f"--var must be NAME=VALUE, got {v!r}")
            continue
        name, _, value = v.partition("=")
        builtin_vars[name.strip()] = value

    # Pre-clean: uninstall every plugin in our marketplace that might be left
    # over from a prior crashed run. Tests that install/uninstall side plugins
    # (e.g. cdxctl-plugin-install installs data-gouv-fr) fail incorrectly when
    # the side plugin is already installed and AI sees "already installed".
    r.log("Pre-clean: uninstalling any stale plugins…")
    for p in mkt_plugins_by_name.values():
        try:
            client.uninstall_plugin(p["id"])
        except RuntimeError:
            pass

    try:
        for name in plugin_names:
            plugin = mkt_plugins_by_name.get(name)
            if plugin is None:
                r.skip(f"{name}: not found in marketplace listing")
                continue
            test_plugin(client, plugin, args, transcript_dir, r, builtin_vars)
    finally:
        remove_marketplace(client, our_mkt["id"], r)

    return r.summary()


if __name__ == "__main__":
    sys.exit(main())
