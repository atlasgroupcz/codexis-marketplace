"""Render a chat session to a Markdown artifact for failure debugging."""

from __future__ import annotations

import json
from pathlib import Path


def render_transcript(test_name: str, steps: list[dict]) -> str:
    """Return Markdown describing each step (prompt, tool calls, text, pass/fail)."""
    lines: list[str] = [f"# {test_name}", ""]
    for i, step in enumerate(steps, start=1):
        status = step.get("status", "UNKNOWN")
        lines.append(f"## Step {i} — {status}")
        lines.append(f"User: {step.get('prompt', '')}")
        result = step.get("result") or {}
        for tc in result.get("tool_calls") or []:
            input_json = json.dumps(tc.get("input"), ensure_ascii=False)
            output_json = json.dumps(tc.get("output"), ensure_ascii=False)
            lines.append(f"Tool call: {tc.get('name')}({input_json})")
            lines.append(f"  → {output_json}")
        text = result.get("text") or ""
        if text:
            lines.append(f"Assistant: {text}")
        if status == "FAIL" and step.get("error"):
            lines.append("")
            lines.append(f"**Assertion failed:** {step['error']}")
        lines.append("")
    return "\n".join(lines)


def write_transcript(out_dir: Path, plugin: str, test_name: str,
                     steps: list[dict]) -> Path:
    """Write transcript to <out_dir>/<plugin>/<test_name>.md and return path."""
    target = out_dir / plugin / f"{test_name}.md"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(render_transcript(test_name, steps), encoding="utf-8")
    return target
