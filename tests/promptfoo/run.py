#!/usr/bin/env python3
"""Run promptfoo evals one plugin batch at a time, with parallel evals per batch.

The cdx-daemon truncates the chat's available-skills system prompt at ~9
skills (alphabetical). With too many skills installed, plugins past the cutoff
become invisible to the AI. This script groups configs into batches whose
combined skill count stays under that cap, installs every plugin in the
batch together, and runs all the batch's evals in parallel against the same
daemon.

Per batch:
  1. Uninstall everything not in the batch.
  2. Install every plugin in the batch.
  3. Run setup.py serially for any configs that need fixtures.
  4. Spawn `npx promptfoo eval -c <cfg>` subprocesses in parallel, one per
     config. Each subprocess has its own concurrency=4 inside.
  5. Collect each subprocess's captured output and pass/fail.

Configs not listed in `BATCHES` get their own solo batch — useful for
mutation-heavy plugins where shared state would interfere (e.g. the cdxctl
battery once it comes back).

Marketplace lifecycle mirrors `tests/test-marketplace.py`:
  - At startup: `add_marketplace_idempotent` removes any pre-existing copy
    (matched on the local manifest's name) and adds the requested ref fresh.
  - At end (try/finally): `remove_marketplace` deletes it. The daemon ends
    in the same clean state it started in.

Usage:
    export CDX_EVAL_AUTH_TOKEN=<jwt>
    tests/promptfoo/run.py \\
        --git-url https://gitlab.agrp.dev/profidata/codexis-marketplace.git \\
        --git-ref feat/promptfoo-poc-cdx-at \\
        [video-analyze.config.yaml ocr.config.yaml ...]
"""

from __future__ import annotations

import argparse
import json
import os
import secrets
import subprocess
import sys
from pathlib import Path

import yaml

THIS = Path(__file__).resolve()
REPO_ROOT = THIS.parent.parent.parent
sys.path.insert(0, str(THIS.parent.parent))
from _daemon_client import DaemonClient  # noqa: E402

# Filename → plugin name override. Empty by default — most config files are
# named after their plugin (cdx-at.config.yaml → cdx-at). Use a YAML-level
# `target_plugin: <name>` field for per-config overrides.
CONFIG_OVERRIDE: dict[str, str] = {}

# Configs are grouped into batches: every config in a batch is installed
# together and evaluated in parallel. Daemon's chat system-prompt only shows
# ~9 skills, so each batch's combined skill count must stay under that cap.
# Any config not listed here runs as its own (solo) batch.
BATCHES: list[list[str]] = [
    # 9 single-skill plugins (1 skill each → 9 visible, right at the cap).
    # Configs that share a `target_plugin` (e.g. ares-ico + ares-dph both
    # install the `ares` plugin) install once and run in parallel against
    # the same daemon — the orchestrator deduplicates plugin installs.
    [
        "ares-ico.config.yaml",
        "ares-dph.config.yaml",
        "cdx-at-ogh.config.yaml",
        "cdx-at-abgb.config.yaml",
        "cdx-cz-psp.config.yaml",
        "cdx-cz-spp.config.yaml",
        "cdx-sk-sknus.config.yaml",
        "cdx-sk-oz.config.yaml",
        "data-gouv-fr.config.yaml",
        "ocr.config.yaml",
        "presentation.config.yaml",
        "video-analyze.config.yaml",
    ],
    # codexis ships 6 skills (codexis, codexis-dane, codexis-ucetnictvi,
    # cdxctl, sledovana-judikatura, sledovane-dokumenty) — solo install,
    # but the search + §10 NOZ test rows run in parallel.
    [
        "codexis-search.config.yaml",
        "codexis-noz.config.yaml",
    ],
    # visualization ships 10 skills (visualize + 9 visualize-* variants).
    # The tested skill `visualize-chart` is alphabetically 3rd, so it stays
    # visible despite the cap — solo for safety.
    ["visualization.config.yaml"],
]


def plugin_name_for(cfg_path: Path, cfg: dict) -> str:
    if "target_plugin" in cfg:
        return cfg["target_plugin"]
    if cfg_path.name in CONFIG_OVERRIDE:
        return CONFIG_OVERRIDE[cfg_path.name]
    return cfg_path.name.replace(".config.yaml", "")


def build_client() -> DaemonClient:
    token = os.environ.get("CDX_EVAL_AUTH_TOKEN", "")
    if not token:
        raise SystemExit("CDX_EVAL_AUTH_TOKEN not set (Keycloak access token)")
    graphql_url = os.environ.get(
        "CDX_EVAL_GRAPHQL_URL", "http://localhost:8086/graphql")
    base = (graphql_url[: -len("/graphql")]
            if graphql_url.endswith("/graphql")
            else graphql_url.rstrip("/"))
    return DaemonClient(base, token)


def installed_plugins(client: DaemonClient) -> list[dict]:
    return client.gql_data(
        "query { installedPlugins { id name } }", {})["installedPlugins"]


def marketplace_plugin_id(client: DaemonClient, name: str) -> str:
    """Resolve a plugin name → its MarketplacePlugin id (used by installPlugin).

    InstalledPlugin id and MarketplacePlugin id are different ID formats —
    install takes the marketplace one.
    """
    for mp in client.list_marketplaces():
        for p in mp.get("plugins") or []:
            if p["name"] == name:
                return p["id"]
    raise RuntimeError(f"plugin {name!r} not found in any marketplace")


def isolate_batch(client: DaemonClient, target_names: list[str]) -> None:
    """Ensure ONLY plugins in `target_names` are installed."""
    current = installed_plugins(client)
    target_set = set(target_names)
    for p in current:
        if p["name"] not in target_set:
            try:
                client.uninstall_plugin(p["id"])
                print(f"  - uninstalled {p['name']}", flush=True)
            except Exception as e:
                print(f"  ! uninstall {p['name']} failed: {e}",
                      file=sys.stderr, flush=True)
    current_names = {p["name"] for p in installed_plugins(client)}
    for name in target_names:
        if name in current_names:
            continue
        try:
            mp_id = marketplace_plugin_id(client, name)
            client.install_plugin(mp_id)
            print(f"  + installed {name}", flush=True)
        except Exception as e:
            print(f"  ! install {name} failed: {e}",
                  file=sys.stderr, flush=True)


def run_setup(cfg_path: Path) -> tuple[bool, str]:
    """Run setup.py for a config if needed and return a run_id.

    Returns (ok, run_id). Every config gets a run_id — even ones without a
    `setup:` block — so any `{{ run_id }}` templates in the YAML render to
    a stable value. Without this, configs that put the run_id in a
    `vars.file` path (like visualization) flake based on whether the AI
    happens to render the template literally or substitutes a timestamp.

    `ok=False` means the setup itself errored.
    """
    cfg = yaml.safe_load(cfg_path.read_text())
    if not cfg.get("setup"):
        # No fixtures to upload — just mint a run_id for template rendering.
        return True, secrets.token_hex(2)
    rc = subprocess.run(
        ["python3", str(THIS.parent / "setup.py"), str(cfg_path)]
    ).returncode
    if rc != 0:
        return False, ""
    sidecar = cfg_path.with_suffix(".run_id")
    return True, (sidecar.read_text().strip() if sidecar.exists()
                  else secrets.token_hex(2))


def run_batch(client: DaemonClient,
              batch_cfgs: list[Path]) -> list[tuple[str, bool]]:
    """Install batch's plugins, run all configs in parallel, capture outputs."""
    plugin_names = [plugin_name_for(c, yaml.safe_load(c.read_text()))
                    for c in batch_cfgs]
    label = ", ".join(c.name for c in batch_cfgs)
    print(f"\n=== batch ({len(batch_cfgs)}): {label} ===", flush=True)

    isolate_batch(client, plugin_names)

    # Per-config setup serially (each is a couple of GraphQL calls — fast).
    # Failed setups still produce a (cfg, False) result without spawning eval.
    setup_status: dict[str, tuple[bool, str]] = {}
    for cfg in batch_cfgs:
        setup_status[cfg.name] = run_setup(cfg)
        if not setup_status[cfg.name][0]:
            print(f"  ! setup failed for {cfg.name}",
                  file=sys.stderr, flush=True)

    # Spawn evals in parallel for everything that survived setup.
    procs: list[tuple[Path, subprocess.Popen | None]] = []
    for cfg in batch_cfgs:
        ok, run_id = setup_status[cfg.name]
        if not ok:
            procs.append((cfg, None))
            continue
        cmd = ["npx", "promptfoo", "eval", "-c", str(cfg), "--no-cache"]
        if run_id:
            cmd += ["--var", f"run_id={run_id}"]
        # Promptfoo's Python worker has a hardcoded 5-min timeout per call.
        # We cap our chat poll well below that so chats time out cleanly
        # (provider returns an error result) instead of letting the worker
        # die with an orphaned response file — which freezes the whole batch.
        env = {**os.environ, "CDX_EVAL_POLL_TIMEOUT_S": os.environ.get(
            "CDX_EVAL_POLL_TIMEOUT_S", "270")}
        proc = subprocess.Popen(cmd, cwd=str(cfg.parent),
                                stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT,
                                env=env,
                                text=True)
        procs.append((cfg, proc))
    n_spawned = sum(1 for _, p in procs if p)
    print(f"  > spawned {n_spawned} parallel eval(s); "
          f"waiting for completion...", flush=True)

    # Wait for all and print captured output in submission order — keeps
    # logs readable while still benefiting from parallel execution.
    results: list[tuple[str, bool]] = []
    for cfg, proc in procs:
        if proc is None:
            results.append((cfg.name, False))
            continue
        out, _ = proc.communicate()
        ok_rc = proc.returncode == 0
        sys.stdout.write(f"\n--- {cfg.name} ---\n")
        sys.stdout.write(out)
        sys.stdout.flush()
        results.append((cfg.name, ok_rc))
    return results


def assign_batches(cfgs: list[Path]) -> list[list[Path]]:
    """Group `cfgs` into batches per BATCHES; everything else runs solo."""
    cfg_by_name = {c.name: c for c in cfgs}
    requested = set(cfg_by_name)
    out: list[list[Path]] = []
    consumed: set[str] = set()
    for batch in BATCHES:
        keep = [cfg_by_name[n] for n in batch if n in requested]
        if keep:
            out.append(keep)
            consumed.update(c.name for c in keep)
    for name, cfg in cfg_by_name.items():
        if name not in consumed:
            out.append([cfg])
    return out


def add_marketplace(client: DaemonClient, git_url: str, git_ref: str) -> dict:
    """Add (or replace) the local repo's marketplace at the given git ref.

    Mirrors `tests/test-marketplace.py::add_marketplace`: removes any existing
    marketplace whose name matches `.claude-plugin/marketplace.json`, then
    adds against the requested ref.
    """
    manifest_path = REPO_ROOT / ".claude-plugin" / "marketplace.json"
    manifest = json.loads(manifest_path.read_text())
    print(f"marketplace: {git_url} @ {git_ref} (name: {manifest['name']})",
          flush=True)
    return client.add_marketplace_idempotent(git_url, git_ref, manifest)


def remove_marketplace(client: DaemonClient, marketplace_node_id: str) -> None:
    try:
        client.remove_marketplace(marketplace_node_id)
        print("marketplace removed", flush=True)
    except Exception as e:
        print(f"  ! remove_marketplace failed: {e}",
              file=sys.stderr, flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--git-url", required=True,
                        help="Marketplace git repository URL")
    parser.add_argument("--git-ref", required=True,
                        help="Marketplace git branch or tag")
    parser.add_argument("--plugin", "--plugins", dest="plugins",
                        nargs="+", default=[],
                        help="Plugin name(s) — runs every config whose "
                             "target_plugin matches. E.g. `--plugin ares` "
                             "runs ares-ico + ares-dph.")
    parser.add_argument("configs", nargs="*",
                        help="Specific config files (default: all "
                             "*.config.yaml). Combines with --plugin.")
    args = parser.parse_args()

    here = THIS.parent
    all_cfgs = sorted(here.glob("*.config.yaml"))
    if not args.configs and not args.plugins:
        cfgs = all_cfgs
    else:
        seen: set[Path] = set()
        cfgs: list[Path] = []
        for arg in args.configs:
            p = Path(arg) if Path(arg).is_absolute() else (here / arg)
            if not p.is_file():
                print(f"no such config file: {arg!r}", file=sys.stderr)
                return 2
            if p not in seen:
                seen.add(p); cfgs.append(p)
        if args.plugins:
            cfgs_by_plugin: dict[str, list[Path]] = {}
            for c in all_cfgs:
                name = plugin_name_for(c, yaml.safe_load(c.read_text()))
                cfgs_by_plugin.setdefault(name, []).append(c)
            for name in args.plugins:
                matched = cfgs_by_plugin.get(name, [])
                if not matched:
                    print(f"no configs target plugin {name!r} "
                          f"(known: {sorted(cfgs_by_plugin)})",
                          file=sys.stderr)
                    return 2
                for c in matched:
                    if c not in seen:
                        seen.add(c); cfgs.append(c)
    if not cfgs:
        print("no *.config.yaml found", file=sys.stderr)
        return 2

    client = build_client()
    our_mkt = add_marketplace(client, args.git_url, args.git_ref)
    batches = assign_batches(cfgs)

    results: list[tuple[str, bool]] = []
    try:
        for batch in batches:
            results.extend(run_batch(client, batch))
    finally:
        print("\n=== removing marketplace ===", flush=True)
        remove_marketplace(client, our_mkt["id"])

    passed = sum(1 for _, ok in results if ok)
    print("\n=== SUMMARY ===")
    for name, ok in results:
        print(f"  {'PASS' if ok else 'FAIL'}  {name}")
    print(f"\n{passed}/{len(results)} passed")
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
