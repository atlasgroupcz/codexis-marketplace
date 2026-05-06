"""Custom Python assertions for promptfoo cdx-daemon evals.

Two-axis test design:
  - response axis  → built-in `regex`, `contains`, `llm-rubric`.
  - tool_call axis → this module: did the AI invoke a specific tool with
                     args matching a regex over the JSON-encoded input?

Tool calls reach this module through a sentinel-bracketed JSON suffix
appended to the provider's `output`. Promptfoo's Python wrapper strips
`providerResponse.metadata` from the assertion context, so the output
string is the only reliable channel. See `provider.py`.
"""

from __future__ import annotations

import json
import re

# Must match the constants in provider.py.
TOOL_CALLS_SENTINEL = "<<CDX_EVAL_TOOL_CALLS>>"
TOOL_CALLS_END = "<<CDX_EVAL_END>>"
_SENTINEL_RE = re.compile(
    re.escape(TOOL_CALLS_SENTINEL) + r"(.*?)" + re.escape(TOOL_CALLS_END),
    re.DOTALL,
)


def _extract_tool_calls(output: str) -> list[dict]:
    """Pull the tool_calls JSON out of the output's sentinel suffix."""
    if not isinstance(output, str):
        return []
    m = _SENTINEL_RE.search(output)
    if not m:
        return []
    try:
        data = json.loads(m.group(1))
    except json.JSONDecodeError:
        return []
    return data if isinstance(data, list) else []


def assert_tool_call(output: str, context: dict) -> dict:
    """Pass if any captured tool call matches `tool_name` + `input_regex`.

    Configure via the test's `vars`:
        tool_name:    str — concrete *ToolMessagePart.toolName ("shell",
                      "writeFile", "readFile", "skill", …).
        input_regex:  str — Python regex matched against the
                      JSON-encoded input dict of each candidate call.
    """
    vars_ = (context or {}).get("vars") or {}
    name = vars_.get("tool_name")
    pattern = vars_.get("input_regex")
    if not name:
        return {"pass": False, "score": 0.0,
                "reason": "assert_tool_call: missing 'tool_name' var"}

    tool_calls = _extract_tool_calls(output)
    rgx = re.compile(pattern) if pattern else None

    saw = [tc.get("name") for tc in tool_calls]
    for tc in tool_calls:
        if tc.get("name") != name:
            continue
        if rgx is None:
            return {"pass": True, "score": 1.0,
                    "reason": f"matched {name} call (no input regex)"}
        blob = json.dumps(tc.get("input") or {}, ensure_ascii=False)
        if rgx.search(blob):
            return {"pass": True, "score": 1.0,
                    "reason": f"matched {name} call: {blob[:120]}"}

    return {"pass": False, "score": 0.0,
            "reason": (f"no {name!r} call matched /{pattern}/. "
                       f"Saw tool names: {saw!r}")}


def assert_no_tool_call(output: str, context: dict) -> dict:
    """Pass if NO tool call's input matches the forbidden regex.

    Anti-cheat axis: catches the AI scraping with `curl`/`wget`/raw
    `python -c "import urllib"` etc. when it should be using the
    plugin's binary. Configure via the test's `vars`:
        forbidden_regex:  str — regex matched against the JSON-encoded
                          input dict of every captured tool call.
        forbidden_tool:   optional str — only check calls of this tool
                          name (default: any tool).
    """
    vars_ = (context or {}).get("vars") or {}
    pattern = vars_.get("forbidden_regex")
    if not pattern:
        return {"pass": False, "score": 0.0,
                "reason": "assert_no_tool_call: missing 'forbidden_regex' var"}
    only_name = vars_.get("forbidden_tool")
    rgx = re.compile(pattern)
    tool_calls = _extract_tool_calls(output)
    for tc in tool_calls:
        if only_name and tc.get("name") != only_name:
            continue
        blob = json.dumps(tc.get("input") or {}, ensure_ascii=False)
        if rgx.search(blob):
            return {"pass": False, "score": 0.0,
                    "reason": (f"forbidden {tc.get('name')!r} call matched "
                               f"/{pattern}/: {blob[:200]}")}
    return {"pass": True, "score": 1.0,
            "reason": f"no calls matched forbidden /{pattern}/"}


def assert_tool_count_max(output: str, context: dict) -> dict:
    """Pass if the AI made <= `tool_calls_max` work calls (skill loads excluded).

    This is a "did the AI actually use the right tool" signal, not a budget
    knob. Healthy single-search workflows are 1-2 work calls; a count
    exploding past the cap usually means the AI's intended tool errored
    (binary not on PATH, auth 401, etc.) and it fell back to curl/scraping.
    Pick the cap as a realistic upper bound for the workflow, then trust it
    — when this fires, investigate the daemon/plugin, not the cap.
    """
    vars_ = (context or {}).get("vars") or {}
    max_n = vars_.get("tool_calls_max")
    if max_n is None:
        return {"pass": False, "score": 0.0,
                "reason": "assert_tool_count_max: missing 'tool_calls_max' var"}
    max_n = int(max_n)
    tool_calls = _extract_tool_calls(output)
    work = [tc for tc in tool_calls if tc.get("name") != "skill"]
    if len(work) <= max_n:
        return {"pass": True, "score": 1.0,
                "reason": f"{len(work)} work calls (≤ {max_n})"}
    names = [tc.get("name") for tc in work]
    return {"pass": False, "score": 0.0,
            "reason": f"{len(work)} work calls (> {max_n}). Names: {names}"}


# ---------------------------------------------------------------------------
# State-via-filesystem — deterministic side-effect oracle. Asserts on a
# daemon-visible file/dir AFTER the chat finished (e.g. "the AI saved
# this HTML at /home/codexis/foo.html"). Lazily imports DaemonClient so
# this module stays import-cheap when the eval doesn't use it.
# ---------------------------------------------------------------------------
def assert_state_file(output: str, context: dict) -> dict:
    """Assert on a daemon-visible filesystem entry via getEntry.

    Configure via the test's `vars`:
        file:   str — VM-side path (templated vars already substituted by
                promptfoo via {{ var }} in the YAML).
        type:   "file" | "dir" | "absent" (default: "file").
    """
    import os
    import sys
    from pathlib import Path

    vars_ = (context or {}).get("vars") or {}
    path = vars_.get("file")
    if not path:
        return {"pass": False, "score": 0.0,
                "reason": "assert_state_file: missing 'file' var"}
    kind = vars_.get("type", "file")

    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from _daemon_client import DaemonClient  # noqa
    graphql_url = os.environ.get(
        "CDX_EVAL_GRAPHQL_URL", "http://localhost:8086/graphql")
    base = (graphql_url[: -len("/graphql")] if graphql_url.endswith("/graphql")
            else graphql_url.rstrip("/"))
    client = DaemonClient(base, os.environ.get("CDX_EVAL_AUTH_TOKEN", ""))
    entry = client.get_entry(path)

    if kind == "absent":
        if entry is None:
            return {"pass": True, "score": 1.0,
                    "reason": f"{path} absent (as expected)"}
        return {"pass": False, "score": 0.0,
                "reason": f"{path} unexpectedly exists"}
    if kind == "file":
        if entry and entry.get("isFile"):
            return {"pass": True, "score": 1.0, "reason": f"{path} is a file"}
        return {"pass": False, "score": 0.0,
                "reason": f"{path} missing or not a file"}
    if kind == "dir":
        if entry and entry.get("isDirectory"):
            return {"pass": True, "score": 1.0, "reason": f"{path} is a directory"}
        return {"pass": False, "score": 0.0,
                "reason": f"{path} missing or not a directory"}
    return {"pass": False, "score": 0.0,
            "reason": f"assert_state_file: unknown type {kind!r}"}
