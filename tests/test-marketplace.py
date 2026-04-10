#!/usr/bin/env python3
"""
Codexis Marketplace Acceptance Test

Tests the marketplace against a running cdx-daemon by cloning via GIT source,
then validating the full lifecycle: install/uninstall each plugin, skills, hooks.

Usage:
  python3 tests/test-marketplace.py --daemon URL --token JWT --git-url URL --git-ref REF

Example:
  python3 tests/test-marketplace.py \
    --daemon http://localhost:8086 \
    --token eyJhbGci... \
    --git-url https://gitlab.agrp.dev/profidata/codexis-marketplace.git \
    --git-ref main
"""

from __future__ import annotations

import argparse
import base64
import json
import sys
import urllib.request
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


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
    errors: list[str] = field(default_factory=list)

    def ok(self, msg: str) -> None:
        self.passed += 1
        print(f"{C.G}  ✓{C.N} {msg}")

    def fail(self, msg: str) -> None:
        self.failed += 1
        self.errors.append(msg)
        print(f"{C.R}  ✗{C.N} {msg}")

    def skip(self, msg: str) -> None:
        print(f"{C.Y}  ⊘{C.N} {msg}")

    def section(self, msg: str) -> None:
        print(f"\n{C.BOLD}━━━ {msg} ━━━{C.N}")

    def log(self, msg: str) -> None:
        print(f"{C.B}[TEST]{C.N} {msg}")

    def summary(self) -> int:
        self.section("Results")
        print(f"  {C.G}Passed:  {self.passed}{C.N}")
        print(f"  {C.R}Failed:  {self.failed}{C.N}")
        print()
        if self.errors:
            print(f"{C.R}{C.BOLD}FAILURES:{C.N}")
            for e in self.errors:
                print(f"  {C.R}✗{C.N} {e}")
            print()
            return 1
        print(f"{C.G}{C.BOLD}All checks passed!{C.N}")
        return 0


# ---------------------------------------------------------------------------
# Daemon client
# ---------------------------------------------------------------------------
class DaemonClient:
    def __init__(self, base_url: str, token: str) -> None:
        self.base_url = base_url.rstrip("/")
        self.graphql_url = f"{self.base_url}/graphql"
        self.health_url = f"{self.base_url}/actuator/health"
        self.auth_header = f"Bearer {token}"

    def health_check(self) -> bool:
        try:
            req = urllib.request.Request(self.health_url)
            with urllib.request.urlopen(req, timeout=5) as resp:
                return json.loads(resp.read()).get("status") == "UP"
        except Exception:
            return False

    def gql(self, query: str, variables: dict | None = None) -> dict[str, Any]:
        payload = json.dumps({"query": query, "variables": variables or {}}).encode()
        req = urllib.request.Request(
            self.graphql_url,
            data=payload,
            headers={
                "Content-Type": "application/json",
                "Authorization": self.auth_header,
            },
        )
        with urllib.request.urlopen(req, timeout=120) as resp:
            return json.loads(resp.read())

    def gql_data(self, query: str, variables: dict | None = None) -> dict[str, Any]:
        result = self.gql(query, variables)
        if result.get("errors"):
            msg = result["errors"][0].get("message", str(result["errors"][0]))
            raise RuntimeError(msg)
        return result["data"]

    def get_entry(self, path: str) -> dict | None:
        data = self.gql_data(GET_ENTRY, {"path": path})
        return data.get("getEntry")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
def encode_node_id(type_name: str, identifier: str) -> str:
    return base64.b64encode(f"{type_name}:{identifier}".encode()).decode()


def check_assertions(assertions: list[dict], install_path: str, client: DaemonClient, r: Results) -> None:
    for a in assertions:
        atype = a.get("type", "")
        path = a.get("path", "").replace("$PLUGIN_DIR", install_path)
        entry = client.get_entry(path)

        if atype == "executable":
            if entry and entry.get("isFile") and entry.get("executable"):
                r.ok(f"{path} is executable")
            elif entry and entry.get("isFile"):
                r.fail(f"{path} exists but is not executable")
            else:
                r.fail(f"{path} missing")

        elif atype == "file":
            if entry and entry.get("isFile"):
                r.ok(f"{path} exists")
            else:
                r.fail(f"{path} missing")

        elif atype == "dir":
            if entry and entry.get("isDirectory"):
                r.ok(f"{path}/ exists")
            else:
                r.fail(f"{path}/ missing")

        elif atype == "absent":
            if entry is None:
                r.ok(f"{path} removed")
            else:
                r.fail(f"{path} still exists")


# ---------------------------------------------------------------------------
# GraphQL queries
# ---------------------------------------------------------------------------
ADD_MARKETPLACE = """
mutation AddMarketplace($input: MarketplaceSourceInput!) {
    addMarketplace(input: $input) {
        id name pluginCount error
        plugins {
            id name description
            skills { name fullName }
        }
    }
}
"""

REMOVE_MARKETPLACE = """
mutation RemoveMarketplace($id: ID!) {
    removeMarketplace(id: $id) { id name }
}
"""

LIST_MARKETPLACES = """
{ marketplaces { id name pluginCount error plugins { id name } } }
"""

INSTALL_PLUGIN = """
mutation InstallPlugin($input: PluginInstallInput!) {
    installPlugin(input: $input) {
        id name version
        installLocation { absolutePath }
        installedAt
    }
}
"""

UNINSTALL_PLUGIN = """
mutation UninstallPlugin($input: PluginInstallInput!) {
    uninstallPlugin(input: $input) { id name }
}
"""

SKILLS = """
{ skills { fullName plugin sourceKind } }
"""

GET_ENTRY = """
query GetEntry($path: String!) {
    getEntry(path: $path) {
        name isFile isDirectory isSymlink executable
    }
}
"""


# ---------------------------------------------------------------------------
# Test steps
# ---------------------------------------------------------------------------
def preflight(client: DaemonClient, marketplace_path: Path, r: Results) -> dict:
    r.section("Pre-flight checks")

    manifest_path = marketplace_path / ".claude-plugin" / "marketplace.json"
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
    plugin_names = [p["name"] for p in manifest.get("plugins", [])]
    r.log(f"Expected plugins ({len(plugin_names)}): {', '.join(plugin_names)}")
    return manifest


def add_marketplace(client: DaemonClient, git_url: str, git_ref: str, manifest: dict, r: Results) -> dict:
    r.section("Add marketplace (GIT)")

    mkt_name = manifest["name"]
    mkt_uuid = manifest.get("uuid", "")
    variables: dict[str, Any] = {"input": {"sourceType": "GIT", "gitUrl": git_url, "gitRef": git_ref}}

    r.log(f"Git URL: {git_url}  ref: {git_ref}")

    try:
        data = client.gql_data(ADD_MARKETPLACE, variables)
    except RuntimeError as e:
        if "already configured" in str(e).lower():
            r.log("Marketplace already exists, removing first...")
            try:
                client.gql_data(REMOVE_MARKETPLACE, {"id": encode_node_id("Marketplace", mkt_uuid)})
            except Exception:
                pass
            data = client.gql_data(ADD_MARKETPLACE, variables)
        else:
            r.fail(f"Failed to add marketplace: {e}")
            sys.exit(1)

    our_mkt = next((m for m in data["addMarketplace"] if m["name"] == mkt_name), None)
    if not our_mkt:
        r.fail(f"Marketplace '{mkt_name}' not found in response")
        sys.exit(1)

    r.ok("Marketplace added successfully")
    if our_mkt.get("error"):
        r.fail(f"Marketplace has error: {our_mkt['error']}")
    if our_mkt["name"] == mkt_name:
        r.ok("Marketplace name matches")
    r.log(f"Marketplace reports {our_mkt['pluginCount']} plugins")
    return our_mkt


def validate_listing(client: DaemonClient, manifest: dict, our_mkt: dict, r: Results) -> None:
    r.section("Validate marketplace listing")

    mkt_name = manifest["name"]
    data = client.gql_data(LIST_MARKETPLACES)
    if any(m["name"] == mkt_name for m in data["marketplaces"]):
        r.ok("Marketplace appears in listing")
    else:
        r.fail("Marketplace not found in listing")

    available_names = {p["name"] for p in our_mkt.get("plugins", [])}
    for plugin_def in manifest.get("plugins", []):
        name = plugin_def["name"]
        if name in available_names:
            r.ok(f"Plugin '{name}' listed")
        else:
            r.fail(f"Plugin '{name}' missing from marketplace (check source path)")


def load_expected(plugin_dir: Path) -> dict | None:
    expected_path = plugin_dir / "tests" / "expected.json"
    if expected_path.is_file():
        return json.loads(expected_path.read_text())
    return None


def test_plugin(client: DaemonClient, plugin: dict, marketplace_path: Path, r: Results) -> None:
    name = plugin["name"]
    plugin_id = plugin["id"]
    skills = plugin.get("skills", [])
    local_dir = marketplace_path / "plugins" / name
    expected = load_expected(local_dir)

    r.section(f"Plugin: {name}")
    r.log(f"  ID: {plugin_id}")
    if skills:
        r.log(f"  Skills: {', '.join(s['fullName'] for s in skills)}")

    # --- Install ---
    r.log("Installing...")
    try:
        data = client.gql_data(INSTALL_PLUGIN, {"input": {"id": plugin_id}})
    except RuntimeError as e:
        r.fail(f"Install failed: {e}")
        return

    installed = next((p for p in data["installPlugin"] if p["name"] == name), None)
    if not installed:
        r.fail("Not in installed list after install")
        return
    r.ok("Install succeeded")

    install_path = (installed.get("installLocation") or {}).get("absolutePath", "")
    r.log(f"  Version: {installed.get('version', '?')}, at: {installed.get('installedAt', '?')}")

    # --- Disk checks (via daemon API) ---
    if install_path:
        entry = client.get_entry(install_path)
        if entry and entry.get("isDirectory"):
            r.ok("Install directory exists")
        else:
            r.fail(f"Install directory missing: {install_path}")

        manifest_entry = client.get_entry(f"{install_path}/.claude-plugin/plugin.json")
        if manifest_entry and manifest_entry.get("isFile"):
            r.ok("Plugin manifest present at install path")
        else:
            r.fail("Plugin manifest missing at install path")
    else:
        r.skip("No install path returned")

    # --- Skill validation (checks skills are available after install) ---
    expected_skills = (expected or {}).get("skills", []) or [s["fullName"] for s in skills]
    if expected_skills:
        try:
            skills_data = client.gql_data(SKILLS)
            available = {s["fullName"] for s in skills_data["skills"]}
            for qname in expected_skills:
                if qname in available:
                    r.ok(f"Skill '{qname}' available")
                else:
                    r.fail(f"Skill '{qname}' not available after install")
        except RuntimeError as e:
            r.fail(f"Failed to query skills: {e}")

    # --- Post-install assertions ---
    if expected and "post_install" in expected:
        r.log("Checking post-install assertions...")
        check_assertions(expected["post_install"], install_path, client, r)

    # --- Uninstall ---
    r.log("Uninstalling...")
    try:
        data = client.gql_data(UNINSTALL_PLUGIN, {"input": {"id": plugin_id}})
    except RuntimeError as e:
        r.fail(f"Uninstall failed: {e}")
        return

    if not any(p["name"] == name for p in data["uninstallPlugin"]):
        r.ok("Removed from installed list")
    else:
        r.fail("Still in installed list after uninstall")

    if install_path:
        entry = client.get_entry(install_path)
        if entry is None:
            r.ok("Install directory cleaned up")
        else:
            r.fail(f"Install directory still exists: {install_path}")

    # --- Post-uninstall assertions ---
    if expected and "post_uninstall" in expected:
        r.log("Checking post-uninstall assertions...")
        check_assertions(expected["post_uninstall"], install_path, client, r)


def remove_marketplace(client: DaemonClient, mkt_node_id: str, mkt_name: str, r: Results) -> None:
    r.section("Remove marketplace")
    try:
        client.gql_data(REMOVE_MARKETPLACE, {"id": mkt_node_id})
    except RuntimeError as e:
        r.fail(f"Failed to remove marketplace: {e}")

    data = client.gql_data(LIST_MARKETPLACES)
    if not any(m["name"] == mkt_name for m in data["marketplaces"]):
        r.ok("Marketplace removed from listing")
    else:
        r.fail("Marketplace still in listing after removal")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main() -> int:
    parser = argparse.ArgumentParser(description="Codexis Marketplace Acceptance Test")
    parser.add_argument("--daemon", required=True, help="Daemon URL (e.g. http://localhost:8086)")
    parser.add_argument("--token", required=True, help="JWT token")
    parser.add_argument("--git-url", required=True, help="Git repository URL")
    parser.add_argument("--git-ref", required=True, help="Git branch or tag")
    args = parser.parse_args()

    marketplace_path = Path(__file__).resolve().parent.parent
    client = DaemonClient(args.daemon, args.token)
    r = Results()

    r.log(f"Daemon: {args.daemon}")
    r.log(f"Marketplace: {marketplace_path}")

    manifest = preflight(client, marketplace_path, r)
    our_mkt = add_marketplace(client, args.git_url, args.git_ref, manifest, r)
    validate_listing(client, manifest, our_mkt, r)

    for plugin in our_mkt.get("plugins", []):
        test_plugin(client, plugin, marketplace_path, r)

    remove_marketplace(client, our_mkt["id"], manifest["name"], r)
    return r.summary()


if __name__ == "__main__":
    sys.exit(main())
