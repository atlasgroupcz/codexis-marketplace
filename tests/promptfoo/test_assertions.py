"""Behavior of the promptfoo assertions: chat sources, and files on the VM."""

import json

import pytest

from tests.promptfoo.assertions import (
    CHAT_NODE_END,
    CHAT_NODE_SENTINEL,
    TOOL_CALLS_END,
    TOOL_CALLS_SENTINEL,
    assert_sources,
    assert_state_file,
)

CHAT_NODE_ID = "Q2hhdDpjaGF0LTE"


def _output(
    tool_calls: list[dict], text: str = "answer", chat_node_id: str = CHAT_NODE_ID
) -> str:
    """Build a provider output string the way provider.py builds it."""
    return (
        f"{text}\n\n{TOOL_CALLS_SENTINEL}{json.dumps(tool_calls)}{TOOL_CALLS_END}"
        f"{CHAT_NODE_SENTINEL}{chat_node_id}{CHAT_NODE_END}"
    )


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


def test_state_file_reads_the_isolation_of_the_chat_that_ran(monkeypatch):
    # The daemon has no view of the VM that is not scoped to an isolation, so
    # the assertion is only looking in the right place if the chat the provider
    # ran reaches getEntry.
    asked = []

    class _Daemon:
        def __init__(self, base_url: str, token: str) -> None:
            pass

        def get_entry(self, isolation_id: str, path: str) -> dict:
            asked.append((isolation_id, path))
            return {
                "name": "chart.html",
                "isFile": True,
                "isDirectory": False,
                "isSymlink": False,
            }

    monkeypatch.setattr("tests.harness.daemon_client.DaemonClient", _Daemon)
    monkeypatch.setenv("CDX_EVAL_GRAPHQL_URL", "http://daemon.test/graphql")
    monkeypatch.setenv("CDX_EVAL_AUTH_TOKEN", "token")

    result = assert_state_file(_output([]), _ctx(file="/home/codexis/chart.html"))

    assert result["pass"] is True
    assert asked == [(CHAT_NODE_ID, "/home/codexis/chart.html")]


@pytest.mark.parametrize(
    "output",
    [f"answer\n\n{TOOL_CALLS_SENTINEL}[]{TOOL_CALLS_END}", None],
    ids=["no sentinel", "not text at all"],
)
def test_state_file_fails_when_the_output_carries_no_chat(output):
    # Better a named failure than a request to the daemon that cannot say which
    # files it means.
    result = assert_state_file(output, _ctx(file="/home/codexis/chart.html"))

    assert result["pass"] is False
    assert "chat node id" in result["reason"]
