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
import concurrent.futures
import json
import os
import re
import secrets
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml

THIS = Path(__file__).resolve()
REPO_ROOT = THIS.parent.parent.parent
sys.path.insert(0, str(THIS.parent.parent))  # tests/ — _daemon_client
sys.path.insert(0, str(THIS.parent))         # tests/promptfoo — _arbitration, assertions
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
        "video-analyze.config.yaml",
    ],
    # visualization ships 10 skills (visualize + 9 visualize-* variants).
    # The tested skill `visualize-chart` is alphabetically 3rd, so it stays
    # visible despite the cap — solo for safety.
    ["visualization.config.yaml"],
    # codexis ships 6 skills (codexis, codexis-dane, codexis-ucetnictvi,
    # cdxctl, sledovana-judikatura, sledovane-dokumenty) — solo install,
    # but each test row runs in parallel against that single install.
    # Tests within a single config run serially (Python provider behavior),
    # which is what we want for cdxctl's create→update→delete sequences.
    # Kept last because cdxctl tests are the heaviest + most likely to push
    # past the orchestrator's JWT TTL; finishing the cheap stuff first means
    # a JWT expiry hits only the codexis batch's teardown, not earlier results.
    [
        "codexis-search.config.yaml",
        "codexis-noz.config.yaml",
        "codexis-cdxctl-skill-crud.config.yaml",
        "codexis-cdxctl-skill-from-file.config.yaml",
        "codexis-cdxctl-agent-crud.config.yaml",
        "codexis-cdxctl-agent-from-file.config.yaml",
        "codexis-cdxctl-automation-ai.config.yaml",
        "codexis-cdxctl-automation-cmd.config.yaml",
        "codexis-cdxctl-automation-update.config.yaml",
        "codexis-cdxctl-marketplace.config.yaml",
        "codexis-cdxctl-notifications.config.yaml",
        "codexis-cdxctl-tabular.config.yaml",
    ],
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


# --------------------------------------------------------------------------- #
# Arbitration: the optional LLM-judge "second phase" (see _arbitration.py).
# Only active under --arbitrate; otherwise the runner behaves exactly as before.
# --------------------------------------------------------------------------- #

@dataclass
class ArbCtx:
    arb: Any                       # the _arbitration module
    call: Any                      # call(messages) -> raw judge JSON string
    model: str
    pf_version: str
    floor: float
    cache: dict
    cache_path: Path
    results_dir: Path
    records: list = field(default_factory=list)


def read_promptfoo_version() -> str:
    """Read the pinned promptfoo version from requirements.txt's marker line."""
    req = THIS.parent / "requirements.txt"
    try:
        for line in req.read_text().splitlines():
            m = re.match(r"#\s*promptfoo:\s*(\S+)", line)
            if m:
                return m.group(1)
    except FileNotFoundError:
        pass
    return "unknown"


def _arbitrate_config(cfg: Path, json_path: Path, ctx: ArbCtx) -> tuple[bool, str]:
    """Re-grade one failed config's failing rows; return (final_ok, summary_note)."""
    try:
        result_json = json.loads(json_path.read_text())
    except (OSError, json.JSONDecodeError) as e:
        print(f"  ! arbitration: cannot read {cfg.name} output: {e}",
              file=sys.stderr, flush=True)
        return False, "arbitration: unreadable output"

    outcomes = ctx.arb.arbitrate_failures(
        result_json, cfg.name, call=ctx.call, model=ctx.model,
        pf_version=ctx.pf_version, floor=ctx.floor, cache=ctx.cache)
    if not outcomes:
        return False, "arbitration: no parseable failures"

    tdir = ctx.results_dir / "transcripts"
    tdir.mkdir(parents=True, exist_ok=True)
    flipped = errored = 0
    for case, result in outcomes:
        if result.passed:
            flipped += 1
        if result.flag == "JUDGE_ERROR":
            errored += 1
        (tdir / f"{ctx.arb.transcript_slug(cfg.name, case.test_idx)}.txt").write_text(
            ctx.arb.format_transcript(case, result, result.passed))
        ctx.records.append({
            "config": cfg.name,
            "test_idx": case.test_idx,
            "description": case.description,
            "verdict": result.verdict,
            "confidence": result.confidence,
            "flag": result.flag,
            "passed": result.passed,
            "rationale": result.rationale,
            "failed_assertions": [{"type": a.get("type"), "value": a.get("value")}
                                  for a in case.failed_assertions],
        })
        print(f"    ⚖ {cfg.name} test {case.test_idx}: {result.verdict} "
              f"({result.confidence}) → {'PASS' if result.passed else 'FAIL'}"
              f"{' [' + result.flag + ']' if result.flag else ''}", flush=True)

    final_ok = ctx.arb.recompute_pass(False, outcomes)
    note = f"{len(outcomes)} failure(s), {flipped} overruled→PASS"
    if errored:
        note += f", {errored} judge error(s)"
    return final_ok, note


def run_batch(client: DaemonClient,
              batch_cfgs: list[Path],
              config_concurrency: int,
              arb_ctx: ArbCtx | None = None) -> list[tuple[str, bool, str]]:
    """Install batch's plugins, run configs with bounded parallelism."""
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

    # The chat poll deadline is the only hard limit on a single test: promptfoo's
    # python provider imposes no per-call timeout (only HTTP fetches honor
    # REQUEST_TIMEOUT_MS, which our urllib-based provider does not use). 600s gives
    # slow plugins room to finish — notably video-analyze, whose transcription /
    # cold model load can exceed the old 270s cap — while the provider still
    # returns a clean error result if the deadline is hit. Override via env.
    env = {**os.environ, "CDX_EVAL_POLL_TIMEOUT_S": os.environ.get(
        "CDX_EVAL_POLL_TIMEOUT_S", "600")}

    def _run_one(cfg: Path) -> tuple[Path, str, bool, Path | None]:
        ok, run_id = setup_status[cfg.name]
        if not ok:
            return cfg, "(setup failed — skipped)\n", False, None
        cmd = ["npx", "promptfoo", "eval", "-c", str(cfg), "--no-cache", "--max-concurrency", "1"]
        if run_id:
            cmd += ["--var", f"run_id={run_id}"]
        # Under --arbitrate we need structured results to find failing rows and
        # their failed assertions; capture promptfoo's JSON to a temp file.
        json_path: Path | None = None
        if arb_ctx is not None:
            fd, name = tempfile.mkstemp(suffix=".json", prefix=f"pf-{cfg.stem}-")
            os.close(fd)
            json_path = Path(name)
            cmd += ["--output", str(json_path)]
        proc = subprocess.Popen(cmd, cwd=str(cfg.parent),
                                stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT,
                                env=env,
                                text=True)
        out, _ = proc.communicate()
        return cfg, out, proc.returncode == 0, json_path

    n_eligible = sum(1 for c in batch_cfgs if setup_status[c.name][0])
    workers = max(1, min(config_concurrency, n_eligible)) if n_eligible else 1
    print(f"  > running {n_eligible} config(s) with config-concurrency={workers}...",
          flush=True)

    # Phase A — bounded parallel run. Up to `workers` Popen children alive at
    # any time — the rest queue up. Mitigates VM fs shell-gate lock contention
    # when many chats race to start against the same per-user VM.
    raw: list[tuple[Path, bool, Path | None]] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as ex:
        # submit in declared order; iterate futures in the same order to keep
        # logs deterministic.
        futures = [ex.submit(_run_one, cfg) for cfg in batch_cfgs]
        for fut in futures:
            cfg, out, ok_rc, json_path = fut.result()
            sys.stdout.write(f"\n--- {cfg.name} ---\n")
            sys.stdout.write(out)
            sys.stdout.flush()
            raw.append((cfg, ok_rc, json_path))

    # Phase B — arbitration (serial: keeps the shared judge cache race-free and
    # bounds concurrent judge calls). Only failed configs are re-graded.
    results: list[tuple[str, bool, str]] = []
    for cfg, ok_rc, json_path in raw:
        ok, note = ok_rc, ""
        if arb_ctx is not None and not ok_rc and json_path is not None and json_path.exists():
            ok, note = _arbitrate_config(cfg, json_path, arb_ctx)
        if json_path is not None:
            json_path.unlink(missing_ok=True)
        results.append((cfg.name, ok, note))
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


_ARB_DEFAULT_FLOOR = 0.6


def _build_arb_ctx(args) -> ArbCtx | None:
    """Resolve the judge endpoint and assemble the arbitration context.

    Imported lazily so the default (no --arbitrate) path never depends on the
    arbitration module or its config. Returns None on configuration error
    (already reported to stderr).
    """
    import _arbitration as arb  # noqa: E402  (lazy)

    try:
        base, key, model = arb.resolve_endpoint(os.environ)
    except arb.ArbitrationConfigError as e:
        print(f"--arbitrate: {e}", file=sys.stderr)
        return None

    cache_path = arb.default_cache_path()
    if args.arbitration_cache_clear:
        cache_path.unlink(missing_ok=True)
    floor = (args.arbitration_confidence if args.arbitration_confidence is not None
             else _ARB_DEFAULT_FLOOR)
    print(f"arbitration: judge={model} floor={floor} "
          f"endpoint={base.rstrip('/')}", flush=True)
    return ArbCtx(
        arb=arb,
        call=arb.make_judge_caller(base, key, model),
        model=model,
        pf_version=read_promptfoo_version(),
        floor=floor,
        cache=arb.load_cache(cache_path),
        cache_path=cache_path,
        results_dir=THIS.parent / ".results",
    )


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
    parser.add_argument("--config-concurrency", type=int, default=3,
                        help="Max parallel config subprocesses per batch "
                             "(default: 3). The per-user VM serializes "
                             "chats on a shell-gate lock; setting this too "
                             "high causes 30s lock-timeout failures.")
    parser.add_argument("--arbitrate", action="store_true",
                        help="Second phase: re-grade each deterministically-"
                             "failing row with an LLM judge against the test's "
                             "vars.arbitration criteria. CORRECT (false positive) "
                             "flips the row to PASS; INCORRECT stays FAIL. Requires "
                             "an OpenAI-compatible judge endpoint (OPENAI_BASE_URL "
                             "+ OPENAI_API_KEY, or CODEXIS_*_LITELLM_* fallback).")
    parser.add_argument("--arbitration-confidence", type=float, default=None,
                        help=f"Confidence floor for honoring a CORRECT verdict "
                             f"(default: {_ARB_DEFAULT_FLOOR}).")
    parser.add_argument("--arbitration-cache-clear", action="store_true",
                        help="Wipe the arbitration verdict cache before running.")
    args = parser.parse_args()
    if args.config_concurrency < 1:
        print("--config-concurrency must be >= 1", file=sys.stderr)
        return 2

    arb_ctx = _build_arb_ctx(args) if args.arbitrate else None
    if args.arbitrate and arb_ctx is None:
        return 2  # config error already reported

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

    results: list[tuple[str, bool, str]] = []
    try:
        for batch in batches:
            results.extend(run_batch(client, batch, args.config_concurrency, arb_ctx))
    finally:
        print("\n=== removing marketplace ===", flush=True)
        remove_marketplace(client, our_mkt["id"])

    if arb_ctx is not None:
        arb_ctx.results_dir.mkdir(parents=True, exist_ok=True)
        (arb_ctx.results_dir / "arbitration.json").write_text(
            json.dumps(arb_ctx.records, ensure_ascii=False, indent=2))
        arb_ctx.arb.save_cache(arb_ctx.cache, arb_ctx.cache_path)
        print(f"\narbitration: {len(arb_ctx.records)} verdict(s) → "
              f"{arb_ctx.results_dir}/arbitration.json", flush=True)

    passed = sum(1 for _, ok, _ in results if ok)
    print("\n=== SUMMARY ===")
    for name, ok, note in results:
        suffix = f"  ({note})" if note else ""
        print(f"  {'PASS' if ok else 'FAIL'}  {name}{suffix}")
    print(f"\n{passed}/{len(results)} passed")
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
