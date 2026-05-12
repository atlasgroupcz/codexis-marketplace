"""YAML assertion vocabulary interpreter.

Pure functions. No I/O. Raise AssertionError (or return error-string, where noted)
on failures; caller formats the error message.
"""

from __future__ import annotations

import json
import re
from typing import Any


_VAR_PATTERN = re.compile(r"\{\{\s*(\w+)\s*\}\}")


def substitute(template: str, vars_: dict[str, Any]) -> str:
    """Replace {{ name }} with vars_[name]. Non-string values are JSON-encoded.

    Raises KeyError if a referenced variable is missing.
    """
    def _repl(m: re.Match) -> str:
        key = m.group(1)
        if key not in vars_:
            raise KeyError(f"missing variable: {key!r}")
        val = vars_[key]
        if isinstance(val, str):
            return val
        return json.dumps(val)
    return _VAR_PATTERN.sub(_repl, template)


def _is_regex_marker(v: Any) -> bool:
    return isinstance(v, str) and v.startswith("~/") and v.endswith("/") and len(v) >= 4


def _extract_regex(v: str) -> str:
    return v[2:-1]


def matches_subset(expected: Any, actual: Any, path: str = "") -> str | None:
    """Check that `expected` is a recursive subset of `actual`.

    Returns None on match, or a human-readable error message on mismatch.
    `expected` can contain regex markers (`~/pattern/`) at leaf positions.
    """
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            return f"at {path or '<root>'}: expected dict, got {type(actual).__name__}"
        for k, v in expected.items():
            sub_path = f"{path}.{k}" if path else k
            if k not in actual:
                return f"at {sub_path}: missing key {k!r} in actual"
            err = matches_subset(v, actual[k], sub_path)
            if err:
                return err
        return None
    if isinstance(expected, list):
        if not isinstance(actual, list) or len(actual) < len(expected):
            return f"at {path or '<root>'}: expected list of >= {len(expected)}, got {actual!r}"
        for i, (e, a) in enumerate(zip(expected, actual)):
            err = matches_subset(e, a, f"{path}[{i}]")
            if err:
                return err
        return None
    if _is_regex_marker(expected):
        pattern = _extract_regex(expected)
        if not isinstance(actual, str):
            return f"at {path or '<root>'}: regex {pattern!r} needs string, got {type(actual).__name__}"
        if re.search(pattern, actual):
            return None
        return f"at {path or '<root>'}: regex {pattern!r} did not match {actual!r}"
    if expected == actual:
        return None
    return f"at {path or '<root>'}: value mismatch — expected {expected!r}, got {actual!r}"


class AssertionFailure(AssertionError):
    """Rich test failure raised by run_step_assertions."""


def _substitute_deep(value: Any, vars_: dict) -> Any:
    """Recursively substitute {{var}} in string leaves of a nested structure."""
    if isinstance(value, str):
        return substitute(value, vars_)
    if isinstance(value, dict):
        return {k: _substitute_deep(v, vars_) for k, v in value.items()}
    if isinstance(value, list):
        return [_substitute_deep(v, vars_) for v in value]
    return value


def _assert_output(expect: dict, text: str, vars_: dict) -> None:
    if "output_contains" in expect:
        needle = substitute(expect["output_contains"], vars_)
        if needle not in text:
            raise AssertionFailure(
                f"output_contains failed: {needle!r} not in {text!r}"
            )
    if "output_not_contains" in expect:
        needle = substitute(expect["output_not_contains"], vars_)
        if needle in text:
            raise AssertionFailure(
                f"output_not_contains failed: {needle!r} IS in {text!r}"
            )
    if "output_matches" in expect:
        pattern = substitute(expect["output_matches"], vars_)
        if not re.search(pattern, text):
            raise AssertionFailure(
                f"output_matches failed: /{pattern}/ did not match {text!r}"
            )


def _assert_tool_calls(expected_list: list[dict], actual_calls: list[dict], vars_: dict) -> None:
    # Empty list = strict "no tool calls"
    if not expected_list:
        if actual_calls:
            names = [c.get("name") for c in actual_calls]
            raise AssertionFailure(f"expected no tool calls, got {names}")
        return

    cursor = 0
    for i, expected in enumerate(expected_list):
        want_name = expected.get("name")
        want_input = _substitute_deep(expected.get("input_contains", {}), vars_)
        matched_at = None
        for j in range(cursor, len(actual_calls)):
            call = actual_calls[j]
            if call.get("name") != want_name:
                continue
            err = matches_subset(want_input, call.get("input") or {})
            if err is None:
                matched_at = j
                break
        if matched_at is None:
            raise AssertionFailure(
                f"tool_calls[{i}] ({want_name}): no matching call found "
                f"in actual calls from index {cursor} onward. "
                f"Actual: {actual_calls!r}"
            )
        cursor = matched_at + 1


def run_step_assertions(expect: dict, result: dict, captured: dict) -> None:
    """Run every assertion declared in `expect` against the parsed step `result`."""
    if "tool_calls" in expect:
        _assert_tool_calls(expect["tool_calls"], result["tool_calls"], captured)
    _assert_output(expect, result["text"], captured)
    # capture and judge are handled by the caller (Task 8, Task 9)


_JUDGE_TEMPLATE = """You are grading whether an AI assistant's response satisfies a rubric.

RUBRIC:
{rubric}

ASSISTANT'S FINAL TEXT:
{text}

TOOL CALLS MADE DURING THIS TURN (JSON):
{tool_calls_json}

Reply with ONLY a compact JSON object of the form:
{{"pass": <true|false>, "reason": "<one sentence>"}}
"""


def run_judge(judge_spec: dict, result: dict, client: Any) -> None:
    """Invoke an LLM judge via `client.run_single_shot_chat(prompt) -> str`.

    Raises AssertionFailure if the judge returns pass=false or malformed JSON.
    """
    rubric = judge_spec.get("rubric") or ""
    prompt = _JUDGE_TEMPLATE.format(
        rubric=rubric,
        text=result.get("text", ""),
        tool_calls_json=json.dumps(result.get("tool_calls", []), ensure_ascii=False),
    )
    raw = client.run_single_shot_chat(prompt)
    try:
        verdict = json.loads(raw)
    except json.JSONDecodeError as e:
        raise AssertionFailure(f"judge reply was not JSON: {raw!r}") from e
    if not verdict.get("pass"):
        reason = verdict.get("reason") or "(no reason given)"
        raise AssertionFailure(f"judge rejected: {reason}")


from jsonpath_ng.ext import parse as _jp_parse


# ---------------------------------------------------------------------------
# Three-axis step checks: tool_call + response + state
#
# Each step's `expect:` block may contain any of the three axes. Every axis
# present is asserted; absent axes are skipped. Fail-fast: the first axis to
# fail raises AssertionFailure and the caller aborts the chat.
# ---------------------------------------------------------------------------


def _check_tool_call(spec: dict, tool_calls: list[dict], vars_: dict) -> None:
    """Assert that at least one tool call matches the spec.

    `spec`: {name: str, input_matches?: regex}
    """
    want_name = spec.get("name")
    want_input_regex = spec.get("input_matches")
    if want_input_regex is not None:
        want_input_regex = substitute(want_input_regex, vars_)
    for call in tool_calls:
        if want_name is not None and call.get("name") != want_name:
            continue
        if want_input_regex is not None:
            input_blob = json.dumps(call.get("input"), ensure_ascii=False)
            if not re.search(want_input_regex, input_blob):
                continue
        return  # matched
    raise AssertionFailure(
        f"tool_call axis failed: no call matched {spec!r}. "
        f"Actual calls: {[c.get('name') for c in tool_calls]!r}"
    )


def _check_response(spec: dict, text: str, vars_: dict) -> None:
    """Assert on the assistant's final text.

    `spec`: {matches?: regex, contains?: substring, not_contains?: substring}
    """
    if "matches" in spec:
        pattern = substitute(spec["matches"], vars_)
        if not re.search(pattern, text):
            raise AssertionFailure(
                f"response axis failed: regex /{pattern}/ did not match {text!r}"
            )
    if "contains" in spec:
        needle = substitute(spec["contains"], vars_)
        if needle not in text:
            raise AssertionFailure(
                f"response axis failed: {needle!r} not in {text!r}"
            )
    if "not_contains" in spec:
        needle = substitute(spec["not_contains"], vars_)
        if needle in text:
            raise AssertionFailure(
                f"response axis failed: {needle!r} unexpectedly in {text!r}"
            )


def _check_state_graphql(spec: dict, client: Any, vars_: dict) -> None:
    """Assert on daemon state via a GraphQL query.

    Fields: graphql (str), variables? (dict), jsonpath? (str extractor),
    and any of: contains, not_contains, equals, matches, count.
    Without `jsonpath`, the entire response data dict is treated as a
    single-element list.
    """
    query = substitute(spec["graphql"], vars_)
    gql_vars = {k: substitute(v, vars_) if isinstance(v, str) else v
                for k, v in (spec.get("variables") or {}).items()}
    try:
        data = client.gql_data(query, gql_vars)
    except Exception as e:
        raise AssertionFailure(f"state.graphql: query failed: {e}") from e

    if "jsonpath" in spec:
        expr = _jp_parse(substitute(spec["jsonpath"], vars_))
        extracted = [m.value for m in expr.find(data)]
    else:
        extracted = [data]

    if "count" in spec and len(extracted) != spec["count"]:
        raise AssertionFailure(
            f"state.graphql: count was {len(extracted)}, expected {spec['count']}. "
            f"Extracted: {extracted!r}"
        )
    if "contains" in spec:
        needle = substitute(spec["contains"], vars_)
        if needle not in [str(v) for v in extracted]:
            raise AssertionFailure(f"state.graphql: {needle!r} not in {extracted!r}")
    if "not_contains" in spec:
        needle = substitute(spec["not_contains"], vars_)
        if needle in [str(v) for v in extracted]:
            raise AssertionFailure(f"state.graphql: {needle!r} unexpectedly in {extracted!r}")
    if "equals" in spec:
        expected = spec["equals"]
        if isinstance(expected, str):
            expected = substitute(expected, vars_)
        if not extracted or extracted[0] != expected:
            raise AssertionFailure(
                f"state.graphql: first value was "
                f"{extracted[0] if extracted else 'none'!r}, expected {expected!r}"
            )
    if "matches" in spec:
        pattern = substitute(spec["matches"], vars_)
        if not any(re.search(pattern, str(v)) for v in extracted):
            raise AssertionFailure(
                f"state.graphql: no extracted value matched /{pattern}/. "
                f"Extracted: {extracted!r}"
            )
    if "matches_subset" in spec:
        if not extracted:
            raise AssertionFailure(
                "state.graphql: matches_subset needs at least one extracted value, got none"
            )
        expected_subset = _substitute_deep(spec["matches_subset"], vars_)
        err = matches_subset(expected_subset, extracted[0])
        if err:
            raise AssertionFailure(f"state.graphql: matches_subset failed — {err}")


def _check_state_file(spec: dict, client: Any, vars_: dict) -> None:
    """Assert on a daemon-visible filesystem entry via getEntry.

    Fields: file (path), type (file|dir|absent — default: file).
    Leverages the same getEntry API test-marketplace.py uses.
    """
    path = substitute(spec["file"], vars_)
    kind = spec.get("type", "file")
    entry = client.get_entry(path)
    if kind == "absent":
        if entry is not None:
            raise AssertionFailure(f"state.file: {path} unexpectedly exists")
    elif kind == "file":
        if not entry or not entry.get("isFile"):
            raise AssertionFailure(f"state.file: {path} missing or not a file")
    elif kind == "dir":
        if not entry or not entry.get("isDirectory"):
            raise AssertionFailure(f"state.file: {path} missing or not a directory")
    else:
        raise ValueError(f"state.file: unknown type {kind!r}")


def _check_state_tool_output(spec: dict, result: dict, vars_: dict) -> None:
    """Assert on the raw output of the most recent tool call in the current turn.

    For API-fetcher plugins whose value is the tool's JSON response, not the
    model's summary. Fields: tool_output (either str regex or a dict with
    matches/contains/not_contains).
    """
    tool_calls = result.get("tool_calls") or []
    if not tool_calls:
        raise AssertionFailure("state.tool_output: no tool calls in this turn")
    blob = json.dumps(tool_calls[-1].get("output"), ensure_ascii=False)
    if isinstance(spec, str):
        pattern = substitute(spec, vars_)
        if not re.search(pattern, blob):
            raise AssertionFailure(
                f"state.tool_output: /{pattern}/ did not match last tool's output"
            )
        return
    if "matches" in spec:
        pattern = substitute(spec["matches"], vars_)
        if not re.search(pattern, blob):
            raise AssertionFailure(
                f"state.tool_output: /{pattern}/ did not match last tool's output"
            )
    if "contains" in spec:
        needle = substitute(spec["contains"], vars_)
        if needle not in blob:
            raise AssertionFailure(f"state.tool_output: {needle!r} not in last tool's output")
    if "not_contains" in spec:
        needle = substitute(spec["not_contains"], vars_)
        if needle in blob:
            raise AssertionFailure(f"state.tool_output: {needle!r} unexpectedly in last tool's output")


def _check_state_single(spec: dict, client: Any, result: dict, vars_: dict) -> None:
    if "graphql" in spec:
        _check_state_graphql(spec, client, vars_)
    elif "file" in spec:
        _check_state_file(spec, client, vars_)
    elif "tool_output" in spec:
        _check_state_tool_output(spec["tool_output"], result, vars_)
    else:
        raise ValueError(
            f"state axis needs one of: graphql, file, tool_output. Got keys: {list(spec)!r}"
        )


def _check_state(spec, client: Any, result: dict, vars_: dict) -> None:
    """State axis. `spec` may be a dict (single check) or a list of dicts (all must pass)."""
    checks = spec if isinstance(spec, list) else [spec]
    for check in checks:
        _check_state_single(check, client, result, vars_)


def _check_tool_calls_count(expect: dict, tool_calls: list) -> None:
    """Bound the number of *work* tool calls the model made in this turn.

    `expect` may contain `tool_calls_min` and/or `tool_calls_max` (int).
    Catches an AI that is too passive (min) or too wasteful (max).

    Skill-loading calls (`name == "skill"`) are excluded — loading a skill
    is context gathering the AI should always do, not part of the work
    budget. They still show up in the `tool_call` axis for explicit matching.
    """
    budget = [c for c in tool_calls if c.get("name") != "skill"]
    n = len(budget)
    if "tool_calls_min" in expect and n < expect["tool_calls_min"]:
        raise AssertionFailure(
            f"tool_calls_min failed: got {n} work call(s), expected >= {expect['tool_calls_min']}. "
            f"Names: {[c.get('name') for c in tool_calls]}"
        )
    if "tool_calls_max" in expect and n > expect["tool_calls_max"]:
        raise AssertionFailure(
            f"tool_calls_max failed: got {n} work call(s), expected <= {expect['tool_calls_max']}. "
            f"Names: {[c.get('name') for c in tool_calls]}"
        )


def run_step_checks(expect: dict, result: dict, captured: dict, client: Any) -> None:
    """Run the axes (tool_call, tool_calls_min/max, response, state). Fail fast.

    Skips any axis not present in `expect`. Raises AssertionFailure on mismatch.
    """
    if "tool_call" in expect:
        _check_tool_call(expect["tool_call"], result.get("tool_calls") or [], captured)
    if "tool_calls_min" in expect or "tool_calls_max" in expect:
        _check_tool_calls_count(expect, result.get("tool_calls") or [])
    if "response" in expect:
        _check_response(expect["response"], result.get("text") or "", captured)
    if "state" in expect:
        _check_state(expect["state"], client, result, captured)


def do_graphql_captures(spec: dict | None, client: Any, vars_: dict) -> dict:
    """Run GraphQL queries and capture values via jsonpath into named vars.

    `spec` shape: { "<var_name>": { "graphql": "...", "variables"?: {...}, "jsonpath": "..." } }
    Returns { "<var_name>": captured_value }. Later steps can use `{{ <var_name> }}`
    in prompts, graphql queries, jsonpaths, and variable values.
    """
    out: dict = {}
    for name, sub in (spec or {}).items():
        query = substitute(sub["graphql"], vars_)
        gql_vars = {k: substitute(v, vars_) if isinstance(v, str) else v
                    for k, v in (sub.get("variables") or {}).items()}
        try:
            data = client.gql_data(query, gql_vars)
        except Exception as e:
            raise AssertionFailure(f"capture {name!r}: query failed: {e}") from e
        if "jsonpath" in sub:
            expr = _jp_parse(substitute(sub["jsonpath"], vars_))
            matches = [m.value for m in expr.find(data)]
            if not matches:
                raise AssertionFailure(
                    f"capture {name!r}: jsonpath {sub['jsonpath']!r} matched nothing"
                )
            out[name] = matches[0]
        else:
            out[name] = data
    return out


def apply_captures(captures: dict | None, result: dict) -> dict:
    """Evaluate jsonpath expressions against the step result and return {name: value}.

    Raises AssertionFailure if any jsonpath resolves to nothing.
    """
    if not captures:
        return {}
    out: dict = {}
    for name, path in captures.items():
        try:
            expr = _jp_parse(path)
        except Exception as e:
            raise AssertionFailure(f"capture {name!r}: invalid jsonpath {path!r}: {e}") from e
        matches = [m.value for m in expr.find(result)]
        if not matches:
            raise AssertionFailure(
                f"capture {name!r}: jsonpath {path!r} did not resolve against result"
            )
        out[name] = matches[0]
    return out
