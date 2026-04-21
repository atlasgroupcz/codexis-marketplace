from pathlib import Path
import re
import sys
import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from _assertions import substitute, matches_subset


# --- substitute ---------------------------------------------------

def test_substitute_replaces_single_var():
    assert substitute("id={{ x }}", {"x": "abc"}) == "id=abc"


def test_substitute_handles_no_vars():
    assert substitute("no vars here", {"x": "abc"}) == "no vars here"


def test_substitute_json_encodes_non_strings():
    assert substitute("v={{ x }}", {"x": 42}) == "v=42"


def test_substitute_raises_on_missing_var():
    with pytest.raises(KeyError, match="missing"):
        substitute("hi {{ missing }}", {})


# --- matches_subset ----------------------------------------------

def test_subset_literal_match_passes():
    assert matches_subset({"a": 1, "b": 2}, {"a": 1, "b": 2, "c": 3}) is None


def test_subset_missing_key_fails():
    err = matches_subset({"a": 1}, {"b": 1})
    assert err is not None and "missing key 'a'" in err


def test_subset_value_mismatch_fails():
    err = matches_subset({"a": 1}, {"a": 2})
    assert err is not None and "value mismatch" in err


def test_subset_regex_prefix_passes():
    assert matches_subset({"name": "~/^e2e-/"}, {"name": "e2e-foo"}) is None


def test_subset_regex_prefix_fails():
    err = matches_subset({"name": "~/^e2e-/"}, {"name": "prod-foo"})
    assert err is not None and "regex" in err


def test_subset_nested_dict_recurses():
    assert matches_subset({"a": {"b": 1}}, {"a": {"b": 1, "c": 2}}) is None
    err = matches_subset({"a": {"b": 1}}, {"a": {"b": 2}})
    assert err is not None


from _assertions import run_step_assertions, AssertionFailure


def _result(text="", tool_calls=()):
    return {"text": text, "tool_calls": list(tool_calls)}


# --- output_* ---------------------------------------------------

def test_output_contains_passes():
    run_step_assertions({"output_contains": "hello"}, _result(text="hello world"), {})


def test_output_contains_fails():
    with pytest.raises(AssertionFailure, match="output_contains"):
        run_step_assertions({"output_contains": "missing"}, _result(text="hi"), {})


def test_output_not_contains_passes():
    run_step_assertions({"output_not_contains": "error"}, _result(text="all good"), {})


def test_output_not_contains_fails():
    with pytest.raises(AssertionFailure, match="output_not_contains"):
        run_step_assertions({"output_not_contains": "bad"}, _result(text="bad news"), {})


def test_output_matches_regex_passes():
    run_step_assertions({"output_matches": r"id=\d+"}, _result(text="created id=42"), {})


def test_output_matches_regex_fails():
    with pytest.raises(AssertionFailure, match="output_matches"):
        run_step_assertions({"output_matches": r"^\d+$"}, _result(text="nope"), {})


def test_output_substitution_with_captured_var():
    run_step_assertions(
        {"output_contains": "{{ id }}"}, _result(text="got auto_7"), {"id": "auto_7"},
    )


# --- tool_calls -------------------------------------------------

def test_tool_calls_passes_on_match():
    run_step_assertions(
        {"tool_calls": [{"name": "cdxctl", "input_contains": {"subcommand": "list"}}]},
        _result(tool_calls=[{"name": "cdxctl",
                             "input": {"subcommand": "list", "verbose": True},
                             "output": {"items": []}}]),
        {},
    )


def test_tool_calls_fails_on_missing_call():
    with pytest.raises(AssertionFailure, match="no matching call"):
        run_step_assertions(
            {"tool_calls": [{"name": "cdxctl"}]},
            _result(tool_calls=[{"name": "other", "input": {}, "output": {}}]),
            {},
        )


def test_tool_calls_enforces_order():
    """Second expected must match an actual call after the one matched by the first."""
    with pytest.raises(AssertionFailure):
        run_step_assertions(
            {"tool_calls": [
                {"name": "cdxctl", "input_contains": {"subcommand": "list"}},
                {"name": "cdxctl", "input_contains": {"subcommand": "create"}},
            ]},
            _result(tool_calls=[
                {"name": "cdxctl", "input": {"subcommand": "create"}, "output": {}},
                {"name": "cdxctl", "input": {"subcommand": "list"}, "output": {}},
            ]),
            {},
        )


def test_tool_calls_empty_list_means_no_tools_called():
    run_step_assertions({"tool_calls": []}, _result(tool_calls=[]), {})
    with pytest.raises(AssertionFailure, match="expected no tool calls"):
        run_step_assertions(
            {"tool_calls": []},
            _result(tool_calls=[{"name": "x", "input": {}, "output": {}}]),
            {},
        )


def test_tool_calls_absent_means_no_assertion():
    # No tool_calls key at all = allow any or none
    run_step_assertions({}, _result(tool_calls=[{"name": "x", "input": {}, "output": {}}]), {})
    run_step_assertions({}, _result(tool_calls=[]), {})


def test_tool_calls_substitutes_vars_in_input_contains():
    run_step_assertions(
        {"tool_calls": [{"name": "cdxctl",
                         "input_contains": {"id": "{{ auto_id }}"}}]},
        _result(tool_calls=[{"name": "cdxctl",
                             "input": {"id": "auto_7"}, "output": {}}]),
        {"auto_id": "auto_7"},
    )


from _assertions import apply_captures


def test_capture_extracts_from_tool_output():
    captured = apply_captures(
        {"aid": "$.tool_calls[0].output.id"},
        _result(tool_calls=[{"name": "x", "input": {}, "output": {"id": "auto_9"}}]),
    )
    assert captured == {"aid": "auto_9"}


def test_capture_multiple_values():
    captured = apply_captures(
        {"aid": "$.tool_calls[0].output.id", "first_text": "$.text"},
        _result(text="hello", tool_calls=[
            {"name": "x", "input": {}, "output": {"id": "auto_9"}},
        ]),
    )
    assert captured == {"aid": "auto_9", "first_text": "hello"}


def test_capture_unresolved_path_raises():
    with pytest.raises(AssertionFailure, match="did not resolve"):
        apply_captures(
            {"aid": "$.tool_calls[5].output.id"},
            _result(tool_calls=[]),
        )


def test_capture_no_captures_returns_empty():
    assert apply_captures(None, _result(text="x")) == {}
    assert apply_captures({}, _result(text="x")) == {}


from _assertions import run_judge


class FakeJudgeClient:
    def __init__(self, scripted_reply: str):
        self.scripted_reply = scripted_reply
        self.last_prompt: str | None = None

    def run_single_shot_chat(self, prompt: str) -> str:
        self.last_prompt = prompt
        return self.scripted_reply


def test_judge_passes():
    fake = FakeJudgeClient('{"pass": true, "reason": "matches rubric"}')
    run_judge(
        judge_spec={"rubric": "response must mention zákon"},
        result=_result(text="zákon je..."),
        client=fake,
    )
    assert "zákon" in (fake.last_prompt or "")


def test_judge_fails_with_reason():
    fake = FakeJudgeClient('{"pass": false, "reason": "missing citation"}')
    with pytest.raises(AssertionFailure, match="missing citation"):
        run_judge(
            judge_spec={"rubric": "response must cite Art. 123"},
            result=_result(text="no citation here"),
            client=fake,
        )


def test_judge_rejects_malformed_reply():
    fake = FakeJudgeClient("not json")
    with pytest.raises(AssertionFailure, match="judge reply"):
        run_judge(
            judge_spec={"rubric": "whatever"},
            result=_result(text="x"),
            client=fake,
        )
