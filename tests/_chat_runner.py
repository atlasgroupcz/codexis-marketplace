"""Drive a cdx-daemon chat session and parse the structured response."""

from __future__ import annotations

import json
from typing import Any


def _parse_json_or_str(raw: str) -> Any:
    if raw is None:
        return None
    try:
        return json.loads(raw)
    except (json.JSONDecodeError, TypeError):
        return raw


def parse_assistant_message(msg: dict) -> dict:
    """Parse a ChatMessage node into {text, tool_calls}.

    - Concatenates all TextMessagePart.content into `text`.
    - Collects ToolMessagePart entries (in order) into `tool_calls`, each
      with {name, input (JSON-parsed if possible), output (JSON-parsed if possible)}.
    - Ignores ThinkingMessagePart.
    """
    text_parts: list[str] = []
    tool_calls: list[dict] = []
    for part in msg.get("parts") or []:
        typ = part.get("__typename")
        if typ == "TextMessagePart":
            text_parts.append(part.get("content") or "")
        elif typ == "ToolMessagePart":
            tool_calls.append({
                "name": part.get("toolName"),
                "input": _parse_json_or_str(part.get("input")),
                "output": _parse_json_or_str(part.get("output")),
            })
        # ThinkingMessagePart and unknown types: ignored
    return {"text": "".join(text_parts), "tool_calls": tool_calls}
