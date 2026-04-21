#!/usr/bin/env python3
"""
Codexis Marketplace Acceptance Test

Tests the marketplace against a running cdx-daemon by cloning via GIT source,
then validating the full lifecycle: install/uninstall each plugin, skills, hooks.

Usage:
  python3 tests/test-marketplace.py --daemon URL --token JWT --git-url URL --git-ref REF

Example:
  python3 tests/test-marketplace.py \\
    --daemon http://localhost:8086 \\
    --token eyJhbGci... \\
    --git-url https://gitlab.agrp.dev/profidata/codexis-marketplace.git \\
    --git-ref main
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _daemon_client import DaemonClient
from _output import Results


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
def check_assertions(assertions: list[dict], install_path: str,
                     client: DaemonClient, r: Results) -> None:
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


def load_expected(plugin_dir: Path) -> dict | None:
    expected_path = plugin_dir / "acceptance" / "expected.json"
    if expected_path.is_file():
        return json.loads(expected_path.read_text())
    return None


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


def add_marketplace(client: DaemonClient, git_url: str, git_ref: str,
                    manifest: dict, r: Results) -> dict:
    r.section("Add marketplace (GIT)")
    mkt_name = manifest["name"]
    r.log(f"Git URL: {git_url}  ref: {git_ref}")

    try:
        marketplaces = client.add_marketplace_idempotent(git_url, git_ref, manifest)
    except RuntimeError as e:
        r.fail(f"Failed to add marketplace: {e}")
        sys.exit(1)

    our_mkt = next((m for m in marketplaces if m["name"] == mkt_name), None)
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
    marketplaces = client.list_marketplaces()
    if any(m["name"] == mkt_name for m in marketplaces):
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
        installed_list = client.install_plugin(plugin_id)
    except RuntimeError as e:
        r.fail(f"Install failed: {e}")
        return

    installed = next((p for p in installed_list if p["name"] == name), None)
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
            skills_list = client.list_skills()
            available = {s["fullName"] for s in skills_list}
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
        remaining = client.uninstall_plugin(plugin_id)
    except RuntimeError as e:
        r.fail(f"Uninstall failed: {e}")
        return

    if not any(p["name"] == name for p in remaining):
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
        client.remove_marketplace(mkt_node_id)
    except RuntimeError as e:
        r.fail(f"Failed to remove marketplace: {e}")

    marketplaces = client.list_marketplaces()
    if not any(m["name"] == mkt_name for m in marketplaces):
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
