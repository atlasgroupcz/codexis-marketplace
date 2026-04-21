from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from _chat_runner import parse_assistant_message


def test_parses_text_only_message():
    msg = {
        "__typename": "ChatMessage",
        "id": "m1",
        "status": "READY",
        "parts": [
            {"__typename": "TextMessagePart", "partId": "p1", "content": "Hello "},
            {"__typename": "TextMessagePart", "partId": "p2", "content": "world"},
        ],
    }
    result = parse_assistant_message(msg)
    assert result == {"text": "Hello world", "tool_calls": []}


def test_parses_tool_call_with_json_input_and_output():
    msg = {
        "status": "READY",
        "parts": [
            {
                "__typename": "ToolMessagePart",
                "toolCallId": "tc1",
                "toolName": "cdxctl",
                "input": '{"subcommand":"create","name":"foo"}',
                "output": '{"id":"auto_1","name":"foo"}',
            },
            {"__typename": "TextMessagePart", "partId": "p1", "content": "Done."},
        ],
    }
    result = parse_assistant_message(msg)
    assert result["text"] == "Done."
    assert result["tool_calls"] == [{
        "name": "cdxctl",
        "input": {"subcommand": "create", "name": "foo"},
        "output": {"id": "auto_1", "name": "foo"},
    }]


def test_preserves_tool_call_order():
    msg = {
        "status": "READY",
        "parts": [
            {"__typename": "ToolMessagePart", "toolCallId": "a", "toolName": "t1",
             "input": "{}", "output": "{}"},
            {"__typename": "ToolMessagePart", "toolCallId": "b", "toolName": "t2",
             "input": "{}", "output": "{}"},
            {"__typename": "ToolMessagePart", "toolCallId": "c", "toolName": "t1",
             "input": "{}", "output": "{}"},
        ],
    }
    names = [tc["name"] for tc in parse_assistant_message(msg)["tool_calls"]]
    assert names == ["t1", "t2", "t1"]


def test_ignores_thinking_parts():
    msg = {
        "status": "READY",
        "parts": [
            {"__typename": "ThinkingMessagePart", "partId": "t1",
             "toolCount": 0, "toolChainId": "c1", "thinkingState": "DONE"},
            {"__typename": "TextMessagePart", "partId": "p1", "content": "final"},
        ],
    }
    assert parse_assistant_message(msg) == {"text": "final", "tool_calls": []}


def test_non_json_tool_output_kept_as_string():
    # Some tools return plain-text output, not JSON.
    msg = {
        "status": "READY",
        "parts": [
            {"__typename": "ToolMessagePart", "toolCallId": "x", "toolName": "shell",
             "input": '{"cmd":"ls"}', "output": "file1\nfile2\n"},
        ],
    }
    tc = parse_assistant_message(msg)["tool_calls"][0]
    assert tc["input"] == {"cmd": "ls"}
    assert tc["output"] == "file1\nfile2\n"


import pytest
from _chat_runner import ChatRunner, ChatError


class FakeDaemon:
    """Minimal fake of the GraphQL surface ChatRunner uses."""
    def __init__(self, scripted_messages: list[dict]):
        self._messages = scripted_messages
        self._cursor = -1  # index into _messages for next assistant reply
        self.sent: list[str] = []
        self.chat_node_id = "Q2hhdElkOnh5eg=="  # base64 of "ChatId:xyz"
        self.poll_count = 0

    def new_chat(self, model=None):
        return {"id": self.chat_node_id, "chatId": "xyz", "status": "READY"}

    def send_message(self, chat_id, message):
        self.sent.append(message)
        self._cursor += 1
        return {"chatId": chat_id, "executionId": f"exec-{self._cursor}"}

    def get_chat(self, node_id):
        self.poll_count += 1
        # Pretend status is PROCESSING for the first poll, READY thereafter.
        status = "READY" if self.poll_count >= 2 else "PROCESSING"
        return {
            "id": node_id,
            "chatId": "xyz",
            "status": status,
            "messages": [self._messages[self._cursor]] if self._cursor >= 0 and status == "READY" else [],
        }


def _msg(text="ok", tool_calls=()):
    parts = [
        {"__typename": "ToolMessagePart", "toolCallId": f"tc{i}", "toolName": tc["name"],
         "input": "{}", "output": "{}"}
        for i, tc in enumerate(tool_calls)
    ]
    parts.append({"__typename": "TextMessagePart", "partId": "p", "content": text})
    return {"__typename": "ChatMessage", "id": "m", "status": "READY", "parts": parts}


def test_chat_runner_sends_and_receives_one_turn():
    fake = FakeDaemon([_msg(text="hello back")])
    runner = ChatRunner(fake, poll_interval_s=0, poll_timeout_s=5)
    runner.start()
    result = runner.step("hi")
    assert result["text"] == "hello back"
    assert fake.sent == ["hi"]


def test_chat_runner_preserves_chat_across_steps():
    fake = FakeDaemon([_msg(text="r1"), _msg(text="r2")])
    runner = ChatRunner(fake, poll_interval_s=0, poll_timeout_s=5)
    runner.start()
    r1 = runner.step("a")
    r2 = runner.step("b")
    assert r1["text"] == "r1"
    assert r2["text"] == "r2"
    assert fake.sent == ["a", "b"]


def test_chat_runner_times_out_on_stuck_processing():
    class Stuck(FakeDaemon):
        def get_chat(self, node_id):
            return {"id": node_id, "chatId": "xyz", "status": "PROCESSING", "messages": []}
    fake = Stuck([])
    runner = ChatRunner(fake, poll_interval_s=0, poll_timeout_s=0.01)
    runner.start()
    with pytest.raises(ChatError, match="timeout"):
        runner.step("hi")


def test_chat_runner_raises_on_error_status():
    class Erroring(FakeDaemon):
        def get_chat(self, node_id):
            return {"id": node_id, "chatId": "xyz", "status": "ERROR", "messages": []}
    fake = Erroring([])
    runner = ChatRunner(fake, poll_interval_s=0, poll_timeout_s=5)
    runner.start()
    with pytest.raises(ChatError, match="ERROR"):
        runner.step("hi")
