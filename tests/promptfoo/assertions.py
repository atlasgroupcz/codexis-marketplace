"""Custom Python assertions for promptfoo cdx-daemon evals.

Two-axis test design:
  - response axis  → built-in `regex`, `contains`, `llm-rubric`.
  - tool_call axis → this module: did the AI invoke a specific tool with
                     args matching a regex over the JSON-encoded input?

Why a custom assertion: promptfoo's standard assertions look at the
provider's `output` text. For tool_call checks we need the structured
tool list the provider returns under metadata. We surface that via a
context-walk that tolerates which-version-of-promptfoo this runs under
(provider metadata is exposed under different attribute paths over time).
"""

from __future__ import annotations

import json
import re
from typing import Any


def _walk(node: Any, *path: str) -> Any:
    for key in path:
        if isinstance(node, dict):
            node = node.get(key)
        else:
            return None
    return node


def _get_tool_calls(context: dict) -> list[dict]:
    """Find the provider's tool_calls list in the assertion context.

    Promptfoo has surfaced provider metadata at several paths across
    versions. Try each and return the first list we find.
    """
    for path in (
        ("providerResponse", "metadata", "tool_calls"),
        ("metadata", "tool_calls"),
        ("response", "metadata", "tool_calls"),
        ("test", "metadata", "tool_calls"),
        ("result", "metadata", "tool_calls"),
    ):
        found = _walk(context, *path)
        if isinstance(found, list):
            return found
    return []


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

    tool_calls = _get_tool_calls(context or {})
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


def assert_tool_count_max(output: str, context: dict) -> dict:
    """Pass if the AI made <= `tool_calls_max` work calls (skill loads excluded)."""
    vars_ = (context or {}).get("vars") or {}
    max_n = vars_.get("tool_calls_max")
    if max_n is None:
        return {"pass": False, "score": 0.0,
                "reason": "assert_tool_count_max: missing 'tool_calls_max' var"}
    max_n = int(max_n)
    tool_calls = _get_tool_calls(context or {})
    work = [tc for tc in tool_calls if tc.get("name") != "skill"]
    if len(work) <= max_n:
        return {"pass": True, "score": 1.0,
                "reason": f"{len(work)} work calls (≤ {max_n})"}
    names = [tc.get("name") for tc in work]
    return {"pass": False, "score": 0.0,
            "reason": f"{len(work)} work calls (> {max_n}). Names: {names}"}
