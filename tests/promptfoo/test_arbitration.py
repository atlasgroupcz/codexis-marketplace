"""Unit tests for the arbitration (LLM-judge second phase) module.

No network: the judge HTTP call is injected as a stub. Run with:
    python3 -m pytest tests/promptfoo/test_arbitration.py
"""

from __future__ import annotations

import json

import pytest

import _arbitration as arb
from assertions import TOOL_CALLS_SENTINEL, TOOL_CALLS_END


# --------------------------------------------------------------------------- #
# Fixtures: build a promptfoo --output JSON shaped exactly like the real one
# (captured from `promptfoo eval --output` at 0.121.12).
# --------------------------------------------------------------------------- #

def _output_with_tools(text: str, tool_calls: list) -> str:
    """Mirror provider.py: AI text + sentinel-wrapped tool_calls JSON suffix."""
    return (f"{text}\n\n{TOOL_CALLS_SENTINEL}"
            f"{json.dumps(tool_calls, ensure_ascii=False)}{TOOL_CALLS_END}")


def _row(*, success, vars, output, description, component_results, test_idx=0):
    return {
        "testIdx": test_idx,
        "success": success,
        "score": 1.0 if success else 0.0,
        "vars": vars,
        "response": {"output": output},
        "testCase": {"description": description, "vars": vars, "assert": []},
        "gradingResult": {
            "pass": success,
            "componentResults": component_results,
        },
    }


def _comp(type_, value, passed, reason):
    return {"pass": passed, "score": 1.0 if passed else 0.0,
            "reason": reason, "assertion": {"type": type_, "value": value}}


CURL_CALLS = [
    {"name": "shell", "input": {"command": "ares-cli detail 45274649"}},
    {"name": "shell", "input": {"command": "curl -I https://example.com/x"}},
]


def _failing_result_json():
    failing = _row(
        success=False,
        vars={
            "prompt": "Co je IČO 45274649? Ověř i odkaz.",
            "forbidden_regex": r"(curl|wget)",
            "arbitration": {
                "correct": ["curl/HEAD to verify a URL the tool returned resolves"],
                "incorrect": ["curl to scrape page content instead of the CLI"],
            },
        },
        output=_output_with_tools("ČEZ, a. s., Duhová 2/1444. Odkaz funguje.", CURL_CALLS),
        description="ICO lookup returns canonical name + address",
        component_results=[
            _comp("python", "file://./assertions.py:assert_tool_call", True, "matched shell call"),
            _comp("python", "file://./assertions.py:assert_no_tool_call", False,
                  "forbidden 'shell' call matched /(curl|wget)/: curl -I https://example.com/x"),
        ],
        test_idx=0,
    )
    passing = _row(
        success=True,
        vars={"prompt": "ok"},
        output=_output_with_tools("fine", []),
        description="a passing test",
        component_results=[_comp("regex", "x", True, "ok")],
        test_idx=1,
    )
    return {"results": {"version": 3, "results": [failing, passing]}}


# --------------------------------------------------------------------------- #
# parse_failures
# --------------------------------------------------------------------------- #

def test_parse_failures_returns_only_failing_rows():
    cases = arb.parse_failures(_failing_result_json(), "ares-ico.config.yaml")
    assert len(cases) == 1
    assert cases[0].test_idx == 0


def test_parse_failures_extracts_prompt_description_and_response_text():
    case = arb.parse_failures(_failing_result_json(), "ares-ico.config.yaml")[0]
    assert case.config == "ares-ico.config.yaml"
    assert case.prompt == "Co je IČO 45274649? Ověř i odkaz."
    assert case.description == "ICO lookup returns canonical name + address"
    # The sentinel-wrapped tool_calls suffix must be stripped from the text.
    assert "ČEZ, a. s., Duhová 2/1444." in case.response_text
    assert TOOL_CALLS_SENTINEL not in case.response_text


def test_parse_failures_recovers_tool_calls_from_sentinel():
    case = arb.parse_failures(_failing_result_json(), "c")[0]
    assert [tc["name"] for tc in case.tool_calls] == ["shell", "shell"]
    assert "curl -I" in json.dumps(case.tool_calls)


def test_parse_failures_keeps_only_failed_assertions():
    case = arb.parse_failures(_failing_result_json(), "c")[0]
    assert len(case.failed_assertions) == 1
    fa = case.failed_assertions[0]
    assert fa["type"] == "python"
    assert fa["value"].endswith("assert_no_tool_call")
    assert "forbidden" in fa["reason"]


def test_parse_failures_extracts_arbitration_criteria():
    case = arb.parse_failures(_failing_result_json(), "c")[0]
    assert case.criteria is not None
    assert case.criteria["correct"][0].startswith("curl/HEAD")
    assert case.criteria["incorrect"][0].startswith("curl to scrape")


def test_parse_failures_handles_results_as_bare_list():
    """Defensive: some promptfoo schemas put results at the top level."""
    bare = _failing_result_json()["results"]["results"]
    cases = arb.parse_failures({"results": bare}, "c")
    assert len(cases) == 1


# --------------------------------------------------------------------------- #
# build_judge_messages
# --------------------------------------------------------------------------- #

def _sample_case(criteria=True):
    return arb.FailedCase(
        config="ares-ico.config.yaml",
        test_idx=0,
        description="ICO lookup returns canonical name + address",
        prompt="Co je IČO 45274649? Ověř i odkaz.",
        response_text="ČEZ, a. s., Duhová 2/1444.",
        tool_calls=CURL_CALLS,
        failed_assertions=[{"type": "python",
                            "value": "file://./assertions.py:assert_no_tool_call",
                            "reason": "forbidden curl call"}],
        criteria=({"correct": ["curl to verify a URL the tool returned"],
                   "incorrect": ["curl to scrape content"]} if criteria else None),
    )


def test_build_judge_messages_has_system_and_user_roles():
    msgs = arb.build_judge_messages(_sample_case())
    assert [m["role"] for m in msgs] == ["system", "user"]


def test_build_judge_messages_includes_prompt_response_and_tool_calls():
    user = arb.build_judge_messages(_sample_case())[1]["content"]
    assert "Co je IČO 45274649" in user
    assert "ČEZ, a. s., Duhová 2/1444." in user
    assert "ares-cli detail 45274649" in user
    assert "curl -I" in user


def test_build_judge_messages_includes_failed_assertion_and_criteria():
    user = arb.build_judge_messages(_sample_case())[1]["content"]
    assert "assert_no_tool_call" in user
    assert "curl to verify a URL the tool returned" in user
    assert "curl to scrape content" in user


def test_build_judge_messages_without_criteria_still_builds():
    user = arb.build_judge_messages(_sample_case(criteria=False))[1]["content"]
    assert "Co je IČO 45274649" in user


# --------------------------------------------------------------------------- #
# parse_verdict
# --------------------------------------------------------------------------- #

def test_parse_verdict_clean_json():
    v = arb.parse_verdict('{"verdict": "CORRECT", "confidence": 0.9, "rationale": "ok"}')
    assert v.verdict == "CORRECT"
    assert v.confidence == 0.9
    assert v.rationale == "ok"


def test_parse_verdict_strips_markdown_fences():
    raw = '```json\n{"verdict": "INCORRECT", "confidence": 0.8, "rationale": "scraped"}\n```'
    v = arb.parse_verdict(raw)
    assert v.verdict == "INCORRECT"
    assert v.confidence == 0.8


def test_parse_verdict_uppercases_verdict():
    v = arb.parse_verdict('{"verdict": "correct", "confidence": 0.7, "rationale": "x"}')
    assert v.verdict == "CORRECT"


def test_parse_verdict_rejects_non_json():
    with pytest.raises(ValueError):
        arb.parse_verdict("the answer is correct, trust me")


def test_parse_verdict_rejects_missing_verdict_field():
    with pytest.raises(ValueError):
        arb.parse_verdict('{"confidence": 0.9}')


# --------------------------------------------------------------------------- #
# arbitrate (verdict -> pass/fail mapping, cache, error handling)
# --------------------------------------------------------------------------- #

def _stub(reply, counter=None):
    def call(messages):
        if counter is not None:
            counter["n"] += 1
        return reply
    return call


def test_arbitrate_correct_high_confidence_flips_to_pass():
    r = arb.arbitrate(_sample_case(),
                      call=_stub('{"verdict":"CORRECT","confidence":0.9,"rationale":"verify"}'),
                      model="m", pf_version="v")
    assert r.passed is True
    assert r.verdict == "CORRECT"
    assert r.flag == ""


def test_arbitrate_incorrect_stays_fail():
    r = arb.arbitrate(_sample_case(),
                      call=_stub('{"verdict":"INCORRECT","confidence":0.95,"rationale":"scrape"}'),
                      model="m", pf_version="v")
    assert r.passed is False
    assert r.verdict == "INCORRECT"


def test_arbitrate_correct_low_confidence_stays_fail_and_flags():
    r = arb.arbitrate(_sample_case(),
                      call=_stub('{"verdict":"CORRECT","confidence":0.3,"rationale":"unsure"}'),
                      model="m", pf_version="v", floor=0.6)
    assert r.passed is False
    assert r.flag == "LOW_CONFIDENCE"


def test_arbitrate_judge_error_stays_fail_and_flags():
    def boom(messages):
        raise RuntimeError("HTTP 500")
    r = arb.arbitrate(_sample_case(), call=boom, model="m", pf_version="v")
    assert r.passed is False
    assert r.flag == "JUDGE_ERROR"
    assert "HTTP 500" in r.rationale


def test_arbitrate_malformed_reply_stays_fail_and_flags():
    r = arb.arbitrate(_sample_case(), call=_stub("not json"), model="m", pf_version="v")
    assert r.passed is False
    assert r.flag == "JUDGE_ERROR"


def test_arbitrate_uses_cache_to_avoid_second_call():
    counter = {"n": 0}
    cache = {}
    call = _stub('{"verdict":"CORRECT","confidence":0.9,"rationale":"v"}', counter)
    case = _sample_case()
    r1 = arb.arbitrate(case, call=call, model="m", pf_version="v", cache=cache)
    r2 = arb.arbitrate(case, call=call, model="m", pf_version="v", cache=cache)
    assert counter["n"] == 1
    assert r1.passed == r2.passed is True


# --------------------------------------------------------------------------- #
# cache key + assertion signature + endpoint resolution
# --------------------------------------------------------------------------- #

def test_cache_key_is_deterministic():
    a = arb.cache_key("m", "v", "p", "r", "sig")
    b = arb.cache_key("m", "v", "p", "r", "sig")
    assert a == b


def test_cache_key_changes_with_model_and_version():
    base = arb.cache_key("m", "v", "p", "r", "sig")
    assert arb.cache_key("m2", "v", "p", "r", "sig") != base
    assert arb.cache_key("m", "v2", "p", "r", "sig") != base


def test_assertion_signature_independent_of_key_order():
    a = arb.assertion_signature([{"type": "python", "value": "x", "reason": "ignored"}])
    b = arb.assertion_signature([{"value": "x", "type": "python", "reason": "other"}])
    assert a == b  # reason is not part of the signature


def test_resolve_endpoint_prefers_openai_vars():
    base, key, model = arb.resolve_endpoint({
        "OPENAI_BASE_URL": "http://proxy/v1",
        "OPENAI_API_KEY": "sk-1",
        "CDX_GRADER_MODEL": "gpt-x",
    })
    assert base == "http://proxy/v1"
    assert key == "sk-1"
    assert model == "gpt-x"


def test_resolve_endpoint_falls_back_to_codexis_litellm_vars():
    base, key, _ = arb.resolve_endpoint({
        "CODEXIS_PUBLIC_LITELLM_BASE_URL": "http://litellm/v1",
        "CODEXIS_USER_LITELLM_API_KEY": "sk-2",
    })
    assert base == "http://litellm/v1"
    assert key == "sk-2"


def test_resolve_endpoint_raises_when_unset():
    with pytest.raises(arb.ArbitrationConfigError):
        arb.resolve_endpoint({})


# --------------------------------------------------------------------------- #
# arbitrate_failures (per-row) + recompute_pass (config-level rollup)
# --------------------------------------------------------------------------- #

def test_arbitrate_failures_judges_each_failing_row_only():
    outcomes = arb.arbitrate_failures(
        _failing_result_json(), "ares-ico.config.yaml",
        call=_stub('{"verdict":"CORRECT","confidence":0.9,"rationale":"verify"}'),
        model="m", pf_version="v")
    assert len(outcomes) == 1            # the passing row is not judged
    case, result = outcomes[0]
    assert case.test_idx == 0
    assert result.passed is True


def test_recompute_pass_original_ok_is_pass():
    assert arb.recompute_pass(True, []) is True


def test_recompute_pass_all_failures_flipped_is_pass():
    flipped = [(_sample_case(), arb.ArbitrationResult(True, "CORRECT", 0.9, "x"))]
    assert arb.recompute_pass(False, flipped) is True


def test_recompute_pass_any_unflipped_failure_stays_fail():
    mixed = [
        (_sample_case(), arb.ArbitrationResult(True, "CORRECT", 0.9, "x")),
        (_sample_case(), arb.ArbitrationResult(False, "INCORRECT", 0.9, "real")),
    ]
    assert arb.recompute_pass(False, mixed) is False


def test_recompute_pass_failed_with_nothing_to_arbitrate_stays_fail():
    # e.g. provider crashed and wrote no parseable failing rows.
    assert arb.recompute_pass(False, []) is False


# --------------------------------------------------------------------------- #
# transcript slug (filesystem safety)
# --------------------------------------------------------------------------- #

def test_transcript_slug_sanitizes_unsafe_chars():
    slug = arb.transcript_slug("ares-ico.config.yaml", 0)
    assert slug == "ares-ico.config.yaml__0"
    assert "/" not in arb.transcript_slug("a/b c:d", 2)
    assert " " not in arb.transcript_slug("a/b c:d", 2)


def test_transcript_slug_is_bounded():
    assert len(arb.transcript_slug("x" * 500, 9)) <= 200
