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
