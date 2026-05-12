#!/usr/bin/env python3
"""Pre-test setup runner for promptfoo configs.

Reads a `setup:` top-level block from the YAML config and executes it
against the daemon BEFORE `promptfoo eval` runs. Promptfoo ignores
top-level keys it doesn't recognize, so the same YAML works for both.

Currently supports:
    setup:
      - upload:
          destination: /home/codexis/ee-foo
          files:
            - path: ocr-test.png
              source: ../fixtures/ocr-test.png   # relative to YAML dir

Usage:
    export CDX_EVAL_AUTH_TOKEN=<jwt>
    tests/promptfoo/setup.py tests/promptfoo/ocr.config.yaml

`{{ run_id }}` in destination/path values is replaced by a stable per-
invocation ID so re-running the suite doesn't collide with prior fixtures.
"""

from __future__ import annotations

import os
import re
import secrets
import sys
from pathlib import Path

import yaml

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from _daemon_client import DaemonClient  # noqa: E402

_VAR = re.compile(r"\{\{\s*(\w+)\s*\}\}")


def _substitute(template: str, vars_: dict) -> str:
    return _VAR.sub(lambda m: str(vars_.get(m.group(1), m.group(0))), template)


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: setup.py <config.yaml>", file=sys.stderr)
        return 2
    cfg_path = Path(sys.argv[1]).resolve()
    cfg = yaml.safe_load(cfg_path.read_text())
    setup = cfg.get("setup") or []
    if not setup:
        print(f"[setup.py] no `setup:` block in {cfg_path.name} — nothing to do.")
        return 0

    token = os.environ.get("CDX_EVAL_AUTH_TOKEN")
    if not token:
        print("[setup.py] CDX_EVAL_AUTH_TOKEN env var required.", file=sys.stderr)
        return 1
    graphql_url = os.environ.get(
        "CDX_EVAL_GRAPHQL_URL", "http://localhost:8086/graphql"
    )
    base = (graphql_url[: -len("/graphql")] if graphql_url.endswith("/graphql")
            else graphql_url.rstrip("/"))
    client = DaemonClient(base, token)

    # Stable run_id per invocation — same convention as the legacy runner,
    # so fixtures don't collide across reruns.
    run_id = secrets.token_hex(2)
    vars_ = {"run_id": run_id}
    print(f"[setup.py] run_id={run_id}")

    for i, action in enumerate(setup):
        if "upload" not in action:
            print(f"[setup.py] action[{i}]: unknown kind, skipping: {list(action)!r}")
            continue
        block = action["upload"]
        destination = _substitute(block["destination"], vars_)
        files: list[tuple[str, bytes | str]] = []
        for f in block.get("files") or []:
            rel = _substitute(f["path"], vars_)
            if "content" in f:
                content: bytes | str = _substitute(f["content"], vars_)
            elif "source" in f:
                src = (cfg_path.parent / _substitute(f["source"], vars_)).resolve()
                content = src.read_bytes()
            else:
                print(f"[setup.py] action[{i}] file {rel!r}: needs `content` or `source`")
                return 1
            files.append((rel, content))
        client.upload_folder(destination, files)
        print(f"[setup.py] uploaded {len(files)} file(s) → {destination}")

    # Emit run_id so callers can chain it into promptfoo via --var or env.
    rc_path = cfg_path.with_suffix(".run_id")
    rc_path.write_text(run_id)
    print(f"[setup.py] run_id written to {rc_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
