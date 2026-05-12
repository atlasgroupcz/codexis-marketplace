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


def assert_tool_count_min(output: str, context: dict) -> dict:
    """Pass if the AI made >= `tool_calls_min` work calls (skill loads excluded).

    Used for tests that need to confirm the AI actually did *some* work
    (e.g. cdxctl skill update must run at least one shell call) — distinct
    from `assert_tool_call` which checks a regex against a specific call.
    """
    vars_ = (context or {}).get("vars") or {}
    min_n = vars_.get("tool_calls_min")
    if min_n is None:
        return {"pass": False, "score": 0.0,
                "reason": "assert_tool_count_min: missing 'tool_calls_min' var"}
    min_n = int(min_n)
    tool_calls = _extract_tool_calls(output)
    work = [tc for tc in tool_calls if tc.get("name") != "skill"]
    if len(work) >= min_n:
        return {"pass": True, "score": 1.0,
                "reason": f"{len(work)} work calls (≥ {min_n})"}
    return {"pass": False, "score": 0.0,
            "reason": f"{len(work)} work calls (< {min_n})"}


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


def assert_state_graphql(output: str, context: dict) -> dict:
    """Run a GraphQL query against the daemon, assert on the result.

    The deterministic side-effect oracle for state-mutating tests
    (cdxctl skill/agent/automation/notification CRUD): after the chat,
    query the daemon for the entity that should now exist (or be gone)
    and check its fields. Mirrors the legacy harness's `state.graphql`
    block.

    Configure via the test's `vars`:
        graphql:        str — query body (promptfoo already substitutes
                        `{{ run_id }}` / other vars in the YAML).
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
        subset:         dict — first extracted dict contains all key→value
                        pairs (legacy `matches_subset`).
    """
    import os
    import re as _re
    import sys
    from pathlib import Path

    vars_ = (context or {}).get("vars") or {}
    query = vars_.get("graphql")
    if not query:
        return {"pass": False, "score": 0.0,
                "reason": "assert_state_graphql: missing 'graphql' var"}

    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from _daemon_client import DaemonClient  # noqa
    graphql_url = os.environ.get(
        "CDX_EVAL_GRAPHQL_URL", "http://localhost:8086/graphql")
    base = (graphql_url[: -len("/graphql")] if graphql_url.endswith("/graphql")
            else graphql_url.rstrip("/"))
    client = DaemonClient(base, os.environ.get("CDX_EVAL_AUTH_TOKEN", ""))

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
                    "reason": (f"count {len(extracted)} ≠ {want}; "
                               f"extracted: {extracted!r}")}
    if "contains" in vars_:
        needle = vars_["contains"]
        if not any(needle in str(v) for v in extracted):
            return {"pass": False, "score": 0.0,
                    "reason": f"contains {needle!r} not in {extracted!r}"}
    if "not_contains" in vars_:
        needle = vars_["not_contains"]
        if any(needle in str(v) for v in extracted):
            return {"pass": False, "score": 0.0,
                    "reason": (f"not_contains {needle!r} unexpectedly "
                               f"in {extracted!r}")}
    if "matches" in vars_:
        pattern = vars_["matches"]
        if not any(_re.search(pattern, str(v)) for v in extracted):
            return {"pass": False, "score": 0.0,
                    "reason": (f"no extracted value matched /{pattern}/. "
                               f"Extracted: {extracted!r}")}
    if "equals" in vars_:
        want = vars_["equals"]
        if not extracted or extracted[0] != want:
            return {"pass": False, "score": 0.0,
                    "reason": (f"first extracted "
                               f"{extracted[0] if extracted else 'none'!r} "
                               f"≠ {want!r}")}
    if "subset" in vars_:
        want = vars_["subset"]
        if not extracted:
            return {"pass": False, "score": 0.0,
                    "reason": f"subset check: no extracted values"}
        first = extracted[0]
        if not isinstance(first, dict):
            return {"pass": False, "score": 0.0,
                    "reason": (f"subset check: first extracted is "
                               f"{type(first).__name__}, want dict")}
        missing = {k: v for k, v in (want or {}).items()
                   if first.get(k) != v}
        if missing:
            return {"pass": False, "score": 0.0,
                    "reason": (f"subset mismatch: expected {want!r}, "
                               f"diff {missing!r} in {first!r}")}
    return {"pass": True, "score": 1.0,
            "reason": f"state.graphql ok ({len(extracted)} value(s))"}
