"""Behavior: what the provider smuggles to the assertions through the output.

Promptfoo strips the provider's metadata before an assertion sees it, so the
output string is the whole channel. A round trip is the only thing that catches
the two sides drifting apart.
"""

from tests.promptfoo import provider
from tests.promptfoo.assertions import _chat_node_id, _extract_tool_calls

CHAT_NODE_ID = "Q2hhdDpjaGF0LTE"


class _Runner:
    """A chat that answers without a daemon."""

    def __init__(self, **kwargs) -> None:
        self.chat_node_id = CHAT_NODE_ID

    def start(self) -> None:
        pass

    def step(self, prompt: str) -> dict:
        return {"text": "done", "tool_calls": [{"name": "shell"}]}


def _output(monkeypatch) -> str:
    monkeypatch.setattr(provider, "ChatRunner", _Runner)
    monkeypatch.setattr(provider, "_build_client", lambda: object())
    return provider.call_api("draw a chart", None, None)["output"]


def test_the_output_names_the_chat_the_turn_ran_in(monkeypatch):
    assert _chat_node_id(_output(monkeypatch)) == CHAT_NODE_ID


def test_the_output_still_carries_the_tool_calls(monkeypatch):
    assert _extract_tool_calls(_output(monkeypatch)) == [{"name": "shell"}]


def test_the_answer_comes_first_so_a_regex_assertion_sees_it(monkeypatch):
    assert _output(monkeypatch).startswith("done")
