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
from dataclasses import dataclass, field
from pathlib import Path

import yaml

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _assertions import (
    AssertionFailure,
    apply_captures,
    run_judge,
    run_step_assertions,
    substitute,
)
from _chat_runner import ChatRunner
from _changed_plugins import get_changed_plugins
from _daemon_client import (
    ADD_MARKETPLACE,
    INSTALL_PLUGIN,
    REMOVE_MARKETPLACE,
    UNINSTALL_PLUGIN,
    DaemonClient,
    encode_node_id,
)
from _transcript import write_transcript


# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------
class C:
    R = "\033[0;31m"
    G = "\033[0;32m"
    Y = "\033[1;33m"
    B = "\033[0;36m"
    BOLD = "\033[1m"
    N = "\033[0m"


@dataclass
class Results:
    passed: int = 0
    failed: int = 0
    skipped: int = 0
    errors: list[str] = field(default_factory=list)

    def ok(self, msg: str) -> None:
        self.passed += 1
        print(f"{C.G}  ✓{C.N} {msg}")

    def fail(self, msg: str) -> None:
        self.failed += 1
        self.errors.append(msg)
        print(f"{C.R}  ✗{C.N} {msg}")

    def skip(self, msg: str) -> None:
        self.skipped += 1
        print(f"{C.Y}  ⊘{C.N} {msg}")

    def section(self, msg: str) -> None:
        print(f"\n{C.BOLD}━━━ {msg} ━━━{C.N}")

    def log(self, msg: str) -> None:
        print(f"{C.B}[E2E]{C.N} {msg}")

    def summary(self) -> int:
        self.section("Results")
        print(f"  {C.G}Passed:  {self.passed}{C.N}")
        print(f"  {C.R}Failed:  {self.failed}{C.N}")
        print(f"  {C.Y}Skipped: {self.skipped}{C.N}")
        if self.errors:
            print(f"\n{C.R}{C.BOLD}FAILURES:{C.N}")
            for e in self.errors:
                print(f"  {C.R}✗{C.N} {e}")
            print()
            return 1
        print(f"\n{C.G}{C.BOLD}All plugin e2e tests passed!{C.N}")
        return 0


# ---------------------------------------------------------------------------
# Plugin selection
# ---------------------------------------------------------------------------
MARKETPLACE_ROOT = Path(__file__).resolve().parent.parent


def _all_plugins_with_e2e() -> list[str]:
    plugins_dir = MARKETPLACE_ROOT / "plugins"
    if not plugins_dir.is_dir():
        return []
    return sorted(
        p.name for p in plugins_dir.iterdir()
        if p.is_dir() and (p / "acceptance" / "e2e").is_dir()
    )


def _yaml_files_for(plugin: str) -> list[Path]:
    d = MARKETPLACE_ROOT / "plugins" / plugin / "acceptance" / "e2e"
    if not d.is_dir():
        return []
    return sorted(d.glob("*.yaml"))


def pick_plugins(args: argparse.Namespace, r: Results) -> list[str]:
    if args.only:
        names = sorted({n.strip() for n in args.only.split(",") if n.strip()})
        r.log(f"--only: {', '.join(names) or '(empty)'}")
        return names
    if args.all:
        names = _all_plugins_with_e2e()
        r.log(f"--all: {len(names)} plugin(s) with acceptance/e2e/")
        return names
    # Default: --changed vs --base-ref
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


def add_marketplace(client: DaemonClient, args: argparse.Namespace,
                    manifest: dict, r: Results) -> dict:
    r.section("Add marketplace (GIT)")
    mkt_name = manifest["name"]
    mkt_uuid = manifest.get("uuid", "")
    variables = {
        "input": {"sourceType": "GIT", "gitUrl": args.git_url, "gitRef": args.git_ref}
    }
    r.log(f"Git URL: {args.git_url}  ref: {args.git_ref}")

    try:
        data = client.gql_data(ADD_MARKETPLACE, variables)
    except RuntimeError as e:
        if "already configured" in str(e).lower():
            r.log("Marketplace already exists, removing first…")
            try:
                client.gql_data(REMOVE_MARKETPLACE,
                                {"id": encode_node_id("Marketplace", mkt_uuid)})
            except Exception:
                pass
            data = client.gql_data(ADD_MARKETPLACE, variables)
        else:
            r.fail(f"Failed to add marketplace: {e}")
            sys.exit(1)

    our = next((m for m in data["addMarketplace"] if m["name"] == mkt_name), None)
    if not our:
        r.fail(f"Marketplace {mkt_name!r} not found in response")
        sys.exit(1)
    r.ok("Marketplace added")
    return our


def remove_marketplace(client: DaemonClient, mkt_node_id: str, r: Results) -> None:
    r.section("Remove marketplace")
    try:
        client.gql_data(REMOVE_MARKETPLACE, {"id": mkt_node_id})
        r.ok("Marketplace removed")
    except RuntimeError as e:
        r.fail(f"Failed to remove marketplace: {e}")


# ---------------------------------------------------------------------------
# One YAML = one multi-turn chat test
# ---------------------------------------------------------------------------
def run_yaml(client: DaemonClient, plugin_name: str, yaml_path: Path,
             transcript_dir: Path, args: argparse.Namespace, r: Results) -> None:
    spec = yaml.safe_load(yaml_path.read_text())
    test_name = spec.get("name") or yaml_path.stem

    if spec.get("skip"):
        r.skip(f"{plugin_name}/{test_name}: {spec['skip']}")
        return

    steps = spec.get("steps") or []
    if not steps:
        r.fail(f"{plugin_name}/{test_name}: no steps defined")
        return

    runner = ChatRunner(
        client,
        poll_interval_s=args.poll_interval_s,
        poll_timeout_s=args.poll_timeout_s,
    )
    runner.start()

    captured: dict = {"run_id": secrets.token_hex(2)}
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
            run_step_assertions(expect, result, captured)
            if "judge" in expect:
                run_judge(expect["judge"], result, client)
            captured.update(apply_captures(expect.get("capture"), result))
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
                transcript_dir: Path, r: Results) -> None:
    name = plugin["name"]
    plugin_id = plugin["id"]
    yamls = _yaml_files_for(name)

    r.section(f"Plugin: {name}")
    if not yamls:
        r.skip(f"{name}: no acceptance/e2e/*.yaml files")
        return

    r.log("Installing…")
    try:
        client.gql_data(INSTALL_PLUGIN, {"input": {"id": plugin_id}})
    except RuntimeError as e:
        r.fail(f"{name}: install failed: {e}")
        return

    try:
        for yaml_path in yamls:
            run_yaml(client, name, yaml_path, transcript_dir, args, r)
    finally:
        r.log("Uninstalling…")
        try:
            client.gql_data(UNINSTALL_PLUGIN, {"input": {"id": plugin_id}})
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
    parser.add_argument("--transcript-dir", default="test-results/transcripts",
                        help="Where to write failure transcripts")
    parser.add_argument("--poll-interval-s", type=float, default=2.0,
                        help="Chat polling interval seconds (default: 2)")
    parser.add_argument("--poll-timeout-s", type=float, default=600.0,
                        help="Chat polling timeout seconds (default: 600)")
    args = parser.parse_args()

    transcript_dir = Path(args.transcript_dir)
    if not transcript_dir.is_absolute():
        transcript_dir = MARKETPLACE_ROOT / transcript_dir

    client = DaemonClient(args.daemon, args.token)
    r = Results()

    r.log(f"Daemon: {args.daemon}")
    r.log(f"Marketplace: {MARKETPLACE_ROOT}")

    manifest = preflight(client, r)
    plugin_names = pick_plugins(args, r)

    if not plugin_names:
        r.log("No plugins to test — done.")
        return r.summary()

    our_mkt = add_marketplace(client, args, manifest, r)
    mkt_plugins_by_name = {p["name"]: p for p in our_mkt.get("plugins", [])}

    try:
        for name in plugin_names:
            plugin = mkt_plugins_by_name.get(name)
            if plugin is None:
                r.skip(f"{name}: not found in marketplace listing")
                continue
            test_plugin(client, plugin, args, transcript_dir, r)
    finally:
        remove_marketplace(client, our_mkt["id"], r)

    return r.summary()


if __name__ == "__main__":
    sys.exit(main())
