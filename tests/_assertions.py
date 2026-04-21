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
