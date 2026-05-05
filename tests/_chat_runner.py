"""Drive a cdx-daemon chat session and parse the structured response.

The daemon's chat schema is interface-driven: ToolMessagePart is an interface
with one concrete subtype per tool kind (ShellToolMessagePart, ReadFileTool…,
etc.) carrying its own typed args. This module flattens those typed parts
back into a uniform `{id, name, input, output}` shape so YAML assertions can
stay tool-agnostic and match `input` as a JSON blob.
"""

from __future__ import annotations

import json
import time
from dataclasses import dataclass
from typing import Any


# Typed args the daemon exposes per concrete ToolMessagePart subtype. We pull
# these field names off the part and pack them into a synthetic input dict
# keyed identically to what the AI saw at call time.
_TOOL_PART_INPUT_FIELDS: dict[str, tuple[str, ...]] = {
    "ShellToolMessagePart": ("command", "note"),
    "ReadFileToolMessagePart": ("path", "offset", "limit"),
    "WriteFileToolMessagePart": ("path", "content"),
    "EditFileToolMessagePart": ("filePath", "oldString", "newString", "replaceAll"),
    "SkillToolMessagePart": ("skill", "resolvedSkillName"),
    "SpawnAgentToolMessagePart": ("subagentType", "prompt", "note", "maxTurns"),
    "ExtractToolMessagePart": ("path", "query", "schemaName"),
}


def _extract_tool_input(part: dict) -> dict:
    """Pull the type-specific input fields off a concrete *ToolMessagePart."""
    keys = _TOOL_PART_INPUT_FIELDS.get(part.get("__typename") or "", ())
    return {k: part[k] for k in keys if part.get(k) is not None}


def _flatten_tool_output(output: dict | None) -> Any:
    """Flatten the ToolOutput union into a plain text/error string or None."""
    if not output:
        return None
    typ = output.get("__typename")
    if typ == "TextToolOutput":
        return output.get("content")
    if typ == "ErrorToolOutput":
        return {"error": output.get("message")}
    if typ == "ImageToolOutput":
        return {"image": output.get("mimeType")}
    return output


def parse_assistant_message(msg: dict) -> dict:
    """Parse an AiChatMessage into {text, tool_calls}.

    - Concatenates TextMessagePart.content into `text`.
    - Each *ToolMessagePart variant becomes one entry in `tool_calls` with
      {id, name, input, output} — `input` is a dict of the typed args the
      daemon recorded for that tool kind.
    - Ignores ReasoningMessagePart (model thinking) and unknown part types.
    """
    text_parts: list[str] = []
    tool_calls: list[dict] = []
    for part in msg.get("parts") or []:
        typ = part.get("__typename") or ""
        if typ == "TextMessagePart":
            text_parts.append(part.get("content") or "")
        elif typ.endswith("ToolMessagePart"):
            tool_calls.append({
                "id": part.get("toolCallId"),
                "name": part.get("toolName"),
                "input": _extract_tool_input(part),
                "output": _flatten_tool_output(part.get("output")),
            })
        # ReasoningMessagePart, unknown types: ignored
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

    def start(self) -> None:
        info = self.client.new_chat()
        self.chat_node_id = info["id"]

    def step(self, prompt: str) -> dict:
        """Send one user message and return parsed {text, tool_calls}.

        The model's tool activity for the turn lives on AiChatMessage.toolChain
        (a sibling node) — we merge those parts back into the turn's tool_calls
        so YAML assertions see the complete picture.
        """
        if self.chat_node_id is None:
            raise ChatError("ChatRunner.start() not called")
        self.client.send_message(self.chat_node_id, prompt)
        deadline = time.monotonic() + self.poll_timeout_s
        while True:
            chat = self.client.get_chat(self.chat_node_id)
            status = chat.get("status")
            if status == "ERROR":
                raise ChatError(f"Chat ended in ERROR state: {chat}")
            if status == "READY":
                msgs = chat.get("messages") or []
                if msgs:
                    return self._parse_turn(msgs[-1])
            if time.monotonic() >= deadline:
                raise ChatError(f"Chat poll timeout after {self.poll_timeout_s}s")
            if self.poll_interval_s > 0:
                time.sleep(self.poll_interval_s)

    def _parse_turn(self, last_msg: dict) -> dict:
        result = parse_assistant_message(last_msg)
        chain = last_msg.get("toolChain") or {}
        for chain_msg in chain.get("messages") or []:
            chain_result = parse_assistant_message(chain_msg)
            result["tool_calls"].extend(chain_result["tool_calls"])
        return result
