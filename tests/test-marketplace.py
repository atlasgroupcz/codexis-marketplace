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
    # cdx guest VM runs as the `codexis` user; install_path looks like
    # /home/codexis/.cdx/plugins/<id> so we treat /home/codexis as $HOME.
    home_dir = "/home/codexis"
    for a in assertions:
        atype = a.get("type", "")
        path = (
            a.get("path", "")
            .replace("$PLUGIN_DIR", install_path)
            .replace("$HOME", home_dir)
        )
        entry = client.get_entry(path)

        if atype == "file":
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


def parse_env_file(content: str) -> dict[str, str]:
    """Parse a KEY=VALUE .env file. Empty lines + # comments ignored.
    Whitespace is stripped from both sides of the value but preserved within."""
    out: dict[str, str] = {}
    for raw in content.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, value = line.partition("=")
        out[key.strip()] = value.strip()
    return out


def check_env_vars(env: dict[str, str], required: list[str], plugin_name: str,
                   r: Results) -> None:
    """Verify each declared env var is present and non-empty in the daemon env."""
    for key in required:
        if key not in env:
            r.fail(f"{plugin_name}: env var {key!r} missing from daemon .env")
        elif not env[key]:
            r.fail(f"{plugin_name}: env var {key!r} present but empty")
        else:
            r.ok(f"{plugin_name}: env var {key!r} set")


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
        our_mkt = client.add_marketplace_idempotent(git_url, git_ref, manifest)
    except RuntimeError as e:
        r.fail(f"Failed to add marketplace: {e}")
        sys.exit(1)

    if not our_mkt or our_mkt.get("name") != mkt_name:
        r.fail(f"Marketplace '{mkt_name}' not in response: {our_mkt!r}")
        sys.exit(1)

    r.ok("Marketplace added successfully")
    if our_mkt.get("error"):
        r.fail(f"Marketplace has error: {our_mkt['error']}")
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


def test_plugin(client: DaemonClient, plugin: dict, marketplace_path: Path,
                daemon_env: dict[str, str] | None, r: Results) -> None:
    name = plugin["name"]
    plugin_id = plugin["id"]
    skills = plugin.get("skills", [])
    local_dir = marketplace_path / "plugins" / name
    expected = load_expected(local_dir)

    r.section(f"Plugin: {name}")
    r.log(f"  ID: {plugin_id}")
    if skills:
        r.log(f"  Skills: {', '.join(s['fullName'] for s in skills)}")

    # --- Daemon env vars (declared by plugin in expected.json) ---
    required_env = (expected or {}).get("env_vars") or []
    if required_env:
        if daemon_env is None:
            r.skip(f"{name}: env var check skipped (daemon .env unreadable)")
        else:
            check_env_vars(daemon_env, required_env, name, r)

    # --- Install ---
    r.log("Installing...")
    try:
        installed = client.install_plugin(plugin_id)
    except RuntimeError as e:
        r.fail(f"Install failed: {e}")
        return

    if not installed or installed.get("name") != name:
        r.fail(f"Install returned unexpected payload: {installed!r}")
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
        client.uninstall_plugin(plugin_id)
        r.ok("Uninstall mutation succeeded")
    except RuntimeError as e:
        r.fail(f"Uninstall failed: {e}")
        return

    # The on-disk getEntry check below is the authoritative "still installed?"
    # signal — the daemon now returns just the uninstalled plugin, not the
    # remaining list, so we can't infer membership from the response.

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
    parser.add_argument("--env-path", default="/home/codexis/.cdx/.env",
                        help="VM-side path to the daemon-managed .env file")
    parser.add_argument("--cookie", default="",
                        help="Optional _oauth2_proxy cookie value used to "
                             "auto-refresh the bearer token on 401.")
    parser.add_argument("--oauth2-proxy", default="http://localhost:4182",
                        help="oauth2-proxy URL (default: http://localhost:4182)")
    args = parser.parse_args()

    marketplace_path = Path(__file__).resolve().parent.parent
    refresher = None
    if args.cookie:
        from _daemon_client import make_oauth2_proxy_refresher
        refresher = make_oauth2_proxy_refresher(args.cookie, args.oauth2_proxy)
    client = DaemonClient(args.daemon, args.token, token_refresher=refresher)
    r = Results()

    r.log(f"Daemon: {args.daemon}")
    r.log(f"Marketplace: {marketplace_path}")

    manifest = preflight(client, marketplace_path, r)
    our_mkt = add_marketplace(client, args.git_url, args.git_ref, manifest, r)
    validate_listing(client, manifest, our_mkt, r)

    # Read the daemon-managed .env once; per-plugin checks consult it.
    daemon_env: dict[str, str] | None
    try:
        raw = client.download_file(args.env_path).decode("utf-8", errors="replace")
        daemon_env = parse_env_file(raw)
        r.log(f"Loaded daemon .env: {len(daemon_env)} keys from {args.env_path}")
    except Exception as e:
        daemon_env = None
        r.log(f"Could not read {args.env_path}: {e} (env-var checks will be skipped)")

    for plugin in our_mkt.get("plugins", []):
        test_plugin(client, plugin, marketplace_path, daemon_env, r)

    remove_marketplace(client, our_mkt["id"], manifest["name"], r)
    return r.summary()


if __name__ == "__main__":
    sys.exit(main())
