"""Promptfoo Python provider for cdx-daemon chats.

Reuses the existing DaemonClient + ChatRunner from `tests/`, so the
promptfoo path stays in lockstep with the legacy e2e runner's
schema knowledge (tool-part interface, FileEntry adapter, etc.).

Auth model matches the cdx-daemon `codexis-eval-ops` provider: caller
provides a Keycloak-issued bearer token directly (no oauth2-proxy
cookie magic in the eval path — eval runs are short enough to fit
inside a single token TTL, and CI uses long-lived service tokens).

Promptfoo invokes `call_api(prompt, options, context)` per test row.
We open one chat per call (one user turn → one assistant turn),
return the AI's text as `output` and the structured tool-call list
as `metadata.tool_calls` for downstream assertions.

Env vars (aligned with codexis-eval-ops):
    CDX_EVAL_GRAPHQL_URL     daemon GraphQL URL (full URL, with /graphql);
                             default http://localhost:8086/graphql
    CDX_EVAL_AUTH_TOKEN      Keycloak access token (required)
    CDX_EVAL_POLL_TIMEOUT_S  per-chat poll deadline, default 600
"""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Any

# Reuse the legacy runner's tested daemon abstractions. They already
# speak the latest schema (Chat / AiChatMessage / typed *ToolMessagePart).
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from _daemon_client import DaemonClient  # noqa: E402
from _chat_runner import ChatRunner  # noqa: E402


def _daemon_base_from_graphql(url: str) -> str:
    """`DaemonClient` wants the base URL; promptfoo convention is full GraphQL URL."""
    if url.endswith("/graphql"):
        return url[: -len("/graphql")]
    return url.rstrip("/")


def _build_client() -> DaemonClient:
    graphql_url = os.environ.get(
        "CDX_EVAL_GRAPHQL_URL", "http://localhost:8086/graphql"
    )
    token = os.environ.get("CDX_EVAL_AUTH_TOKEN", "")
    if not token:
        raise RuntimeError(
            "CDX_EVAL_AUTH_TOKEN env var is required (Keycloak access token)."
        )
    return DaemonClient(_daemon_base_from_graphql(graphql_url), token)


def call_api(prompt: str, options: dict | None, context: dict | None) -> dict[str, Any]:
    """One chat → one prompt → one AI turn. Returns promptfoo response shape."""
    timeout = float(os.environ.get("CDX_EVAL_POLL_TIMEOUT_S", "600"))
    runner = ChatRunner(client=_build_client(), poll_interval_s=2.0,
                        poll_timeout_s=timeout)
    try:
        runner.start()
        result = runner.step(prompt)
    except Exception as e:
        return {"output": "", "error": f"{type(e).__name__}: {e}"}

    text = (result.get("text") or "").strip()
    tool_calls = result.get("tool_calls") or []
    return {
        "output": text,
        # Provider metadata is preserved on the response object; assertions
        # (see assertions.py) walk the context to find it across promptfoo
        # versions. Tool calls are summarized for the LLM grader to see.
        "metadata": {
            "tool_calls": tool_calls,
            "tool_calls_summary": _summarize_for_grader(tool_calls),
            "chat_node_id": runner.chat_node_id,
        },
    }


def _summarize_for_grader(tool_calls: list[dict]) -> str:
    """Compact human-readable summary of tool calls — drops big payloads,
    special-cases the `skill` loader. Mirrors codexis-eval-ops' helper so
    the LLM judge can reason about what the AI did without burning tokens
    on raw 50KB skill instruction dumps.
    """
    import json
    lines: list[str] = []
    for tc in tool_calls:
        name = tc.get("name") or "?"
        if name == "skill":
            lines.append(f"[skill] (instructions loaded)")
            continue
        input_blob = json.dumps(tc.get("input") or {}, ensure_ascii=False)[:500]
        out = tc.get("output")
        out_blob = (out if isinstance(out, str)
                    else json.dumps(out, ensure_ascii=False))[:2000]
        lines.append(f"[{name}] input: {input_blob}\n  output: {out_blob}")
    return "\n".join(lines)
