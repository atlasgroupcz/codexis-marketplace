from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from _transcript import render_transcript


def test_transcript_renders_prompt_and_text():
    steps = [{
        "prompt": "hi",
        "result": {"text": "hello", "tool_calls": []},
        "status": "PASS",
    }]
    md = render_transcript("my-test", steps)
    assert "# my-test" in md
    assert "User: hi" in md
    assert "Assistant: hello" in md
    assert "PASS" in md


def test_transcript_renders_tool_call_compactly():
    steps = [{
        "prompt": "create",
        "result": {"text": "done",
                   "tool_calls": [{"name": "cdxctl",
                                   "input": {"subcommand": "create"},
                                   "output": {"id": "x"}}]},
        "status": "PASS",
    }]
    md = render_transcript("t", steps)
    assert "Tool call: cdxctl" in md
    assert '"subcommand": "create"' in md
    assert '"id": "x"' in md


def test_transcript_marks_failing_step():
    steps = [
        {"prompt": "a", "result": {"text": "ok", "tool_calls": []}, "status": "PASS"},
        {"prompt": "b", "result": {"text": "nope", "tool_calls": []},
         "status": "FAIL", "error": "output_contains failed: 'x' not in 'nope'"},
    ]
    md = render_transcript("t", steps)
    assert "Step 2 — FAIL" in md
    assert "output_contains failed" in md
