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
