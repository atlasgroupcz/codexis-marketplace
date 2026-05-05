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


# ---------------------------------------------------------------------------
# State-via-GraphQL — the deterministic side-effect oracle used by the
# cdxctl tests in the legacy runner. Asserts on daemon state AFTER the
# chat finished (e.g. "an automation named X exists with these fields").
# Lazily imports DaemonClient so this module stays import-cheap when the
# eval doesn't use it.
# ---------------------------------------------------------------------------
def assert_state_graphql(output: str, context: dict) -> dict:
    """Run a GraphQL query against the daemon, assert on the result.

    Configure via the test's `vars`:
        graphql:        str — query body, may reference jinja/promptfoo vars
                        already substituted by promptfoo itself.
        variables:      dict — optional GraphQL variables.
        jsonpath:       str — optional JSONPath expression to extract values.
                        Without it, the whole `data` dict is treated as a
                        one-element list.
        And one or more of:
        contains:       str — pass if any extracted value contains it.
        not_contains:   str — pass if no extracted value contains it.
        matches:        str — Python regex pass-if-any.
        equals:         any — pass if the FIRST extracted value equals.
        count:          int — pass if extracted has exactly this length.
    """
    import os
    import re
    import sys
    from pathlib import Path

    vars_ = (context or {}).get("vars") or {}
    query = vars_.get("graphql")
    if not query:
        return {"pass": False, "score": 0.0,
                "reason": "assert_state_graphql: missing 'graphql' var"}

    # Lazy import — keeps this module light when this assertion isn't used.
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from _daemon_client import DaemonClient  # noqa
    graphql_url = os.environ.get(
        "CDX_EVAL_GRAPHQL_URL", "http://localhost:8086/graphql")
    base = graphql_url[:-len("/graphql")] if graphql_url.endswith("/graphql") else graphql_url.rstrip("/")
    token = os.environ.get("CDX_EVAL_AUTH_TOKEN", "")
    client = DaemonClient(base, token)

    try:
        data = client.gql_data(query, vars_.get("variables") or {})
    except Exception as e:
        return {"pass": False, "score": 0.0,
                "reason": f"assert_state_graphql: query failed: {e}"}

    extracted: list = [data]
    jp = vars_.get("jsonpath")
    if jp:
        try:
            from jsonpath_ng.ext import parse as _jp_parse
            extracted = [m.value for m in _jp_parse(jp).find(data)]
        except Exception as e:
            return {"pass": False, "score": 0.0,
                    "reason": f"assert_state_graphql: jsonpath {jp!r}: {e}"}

    if "count" in vars_:
        want = int(vars_["count"])
        if len(extracted) != want:
            return {"pass": False, "score": 0.0,
                    "reason": f"count {len(extracted)} ≠ {want}; "
                              f"extracted: {extracted!r}"}
    if "contains" in vars_:
        needle = vars_["contains"]
        if not any(needle in str(v) for v in extracted):
            return {"pass": False, "score": 0.0,
                    "reason": f"contains {needle!r} not in {extracted!r}"}
    if "not_contains" in vars_:
        needle = vars_["not_contains"]
        if any(needle in str(v) for v in extracted):
            return {"pass": False, "score": 0.0,
                    "reason": f"not_contains {needle!r} unexpectedly in {extracted!r}"}
    if "matches" in vars_:
        pattern = vars_["matches"]
        if not any(re.search(pattern, str(v)) for v in extracted):
            return {"pass": False, "score": 0.0,
                    "reason": f"no extracted value matched /{pattern}/. "
                              f"Extracted: {extracted!r}"}
    if "equals" in vars_:
        want = vars_["equals"]
        if not extracted or extracted[0] != want:
            return {"pass": False, "score": 0.0,
                    "reason": f"first extracted "
                              f"{extracted[0] if extracted else 'none'!r} ≠ {want!r}"}
    return {"pass": True, "score": 1.0,
            "reason": f"state.graphql ok ({len(extracted)} value(s))"}
