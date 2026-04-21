"""Drive a cdx-daemon chat session and parse the structured response."""

from __future__ import annotations

import json
import time
from dataclasses import dataclass, field
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


class ChatError(RuntimeError):
    pass


@dataclass
class ChatRunner:
    """One instance = one chat session. Call start(), then step(prompt) per turn."""
    client: Any  # duck-typed: needs new_chat, send_message, get_chat
    poll_interval_s: float = 2.0
    poll_timeout_s: float = 600.0
    chat_node_id: str | None = None
    chat_id: str | None = None

    def start(self) -> None:
        info = self.client.new_chat()
        self.chat_node_id = info["id"]
        self.chat_id = info["chatId"]

    def step(self, prompt: str) -> dict:
        """Send one user message and return parsed {text, tool_calls}."""
        if self.chat_node_id is None:
            raise ChatError("ChatRunner.start() not called")
        self.client.send_message(self.chat_id, prompt)
        deadline = time.monotonic() + self.poll_timeout_s
        while True:
            chat = self.client.get_chat(self.chat_node_id)
            status = chat.get("status")
            if status == "ERROR":
                raise ChatError(f"Chat ended in ERROR state: {chat}")
            if status == "READY":
                msgs = chat.get("messages") or []
                if msgs:
                    return parse_assistant_message(msgs[-1])
            if time.monotonic() >= deadline:
                raise ChatError(f"Chat poll timeout after {self.poll_timeout_s}s")
            if self.poll_interval_s > 0:
                time.sleep(self.poll_interval_s)
