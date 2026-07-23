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
    out = {}
    for k in keys:
        v = part.get(k)
        if v is None and k == "content":
            # WriteFileToolMessagePart.content is aliased to fileContent in the
            # chat query (FieldsConflict with TextMessagePart.content: HTML).
            v = part.get("fileContent")
        if v is not None:
            out[k] = v
    return out


def _flatten_tool_output(output: dict | None) -> Any:
    """Flatten ToolOutput.blocks into a plain text/error string or None."""
    if not output:
        return None
    texts: list[str] = []
    for block in output.get("blocks") or []:
        typ = block.get("__typename")
        if typ == "TextToolOutput":
            texts.append(block.get("content") or "")
        elif typ == "ErrorToolOutput":
            return {"error": block.get("message")}
        elif typ == "ImageToolOutput":
            return {"image": block.get("mimeType")}
    return "".join(texts) if texts else None


def parse_assistant_message(msg: dict) -> dict:
    """Parse an AiChatMessage into {text, tool_calls}.

    - Concatenates TextMessagePart.content into `text`.
    - Each *ToolMessagePart variant becomes one entry in `tool_calls` with
      {id, name, input, output} - `input` is a dict of the typed args the
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
                "sources": part.get("sources") or [],
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
        (a sibling node) - we merge those parts back into the turn's tool_calls
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
        # In the new schema, ToolMessageParts already live inline in
        # AiChatMessage.parts. The toolChain is the same entries surfaced via
        # a sibling node - reading both would double-count every tool call.
        return parse_assistant_message(last_msg)
