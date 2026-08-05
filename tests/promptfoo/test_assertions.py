"""Behavior: assert_sources checks the chat sources attached to a turn."""

import json

from tests.promptfoo.assertions import (
    TOOL_CALLS_END,
    TOOL_CALLS_SENTINEL,
    assert_sources,
)


def _output(tool_calls: list[dict], text: str = "answer") -> str:
    """Build a provider output string with tool_calls in the sentinel suffix."""
    return f"{text}\n\n{TOOL_CALLS_SENTINEL}{json.dumps(tool_calls)}{TOOL_CALLS_END}"


def _ctx(**vars_) -> dict:
    return {"vars": vars_}


def test_one_source_meets_default_min():
    out = _output([{"name": "shell", "sources": [{"url": "https://ares.gov.cz/x"}]}])
    assert assert_sources(out, _ctx())["pass"] is True


def test_no_sources_fails_default_min():
    out = _output([{"name": "shell", "sources": []}])
    result = assert_sources(out, _ctx(sources_min=1))
    assert result["pass"] is False
    assert "0 source" in result["reason"]


def test_regex_matches_a_source():
    out = _output(
        [
            {
                "name": "shell",
                "sources": [{"url": "https://ares.gov.cz/ekonomicke-subjekty/x"}],
            }
        ]
    )
    result = assert_sources(
        out, _ctx(sources_regex=r"ares\.gov\.cz/ekonomicke-subjekty")
    )
    assert result["pass"] is True


def test_regex_no_match_fails():
    out = _output(
        [{"name": "shell", "sources": [{"url": "https://evil.example/scrape"}]}]
    )
    result = assert_sources(out, _ctx(sources_regex=r"ares\.gov\.cz"))
    assert result["pass"] is False


def test_sources_flattened_across_tool_calls():
    out = _output(
        [
            {"name": "shell", "sources": [{"url": "https://ares.gov.cz/a"}]},
            {"name": "shell", "sources": [{"url": "https://ares.gov.cz/b"}]},
        ]
    )
    assert assert_sources(out, _ctx(sources_min=2))["pass"] is True
