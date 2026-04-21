# Plugin E2E Testing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a pytest-based e2e test suite that installs each marketplace plugin into a real cdx-daemon, drives a multi-turn chat per plugin, asserts on tool calls and outputs, runs only for plugins changed in the MR, and gates merges in GitLab CI.

**Architecture:** Pure-Python, pytest-based. A `DaemonClient` (extracted from the existing `tests/test-marketplace.py`) handles GraphQL. A `ChatRunner` drives the daemon's chat API (`newChat → sendMessage → poll → parse`). An `Assertions` module interprets a YAML test-case vocabulary (`tool_calls`, `output_contains`, `capture`, optional LLM `judge`). A `ChangedPlugins` module uses `git diff` against a base ref. Pytest collection discovers `plugins/<name>/acceptance/e2e/*.yaml` and creates one test per file. On failure, a Markdown transcript of the full chat is written to `test-results/transcripts/` as a CI artifact.

**Tech Stack:** Python 3.11+, pytest, PyYAML, jsonpath-ng, stdlib `urllib`/`subprocess`. No new runtime deps in the daemon; the suite is purely a test client.

**Design spec:** `docs/superpowers/specs/2026-04-21-plugin-e2e-testing-design.md`

---

## File structure

**New files:**
- `tests/_daemon_client.py` — GraphQL client (marketplace add/remove, plugin install/uninstall, entry queries). Shared with `test-marketplace.py` after refactor.
- `tests/_chat_runner.py` — `newChat → sendMessage → poll → parse assistant message`. Returns `{text, tool_calls: [{name, input, output}]}`.
- `tests/_assertions.py` — Interprets YAML assertion vocabulary against a step result. Pure functions; no I/O.
- `tests/_changed_plugins.py` — `git diff --name-only <base> HEAD` → set of plugin names. Pure logic around a subprocess call.
- `tests/_transcript.py` — Renders chat history to a Markdown artifact on failure.
- `tests/test-plugin-e2e.py` — Pytest collection + per-plugin install/uninstall fixture. Parses CLI args.
- `tests/unit/test_assertions.py` — Unit tests for `_assertions.py`.
- `tests/unit/test_chat_runner.py` — Unit tests for `_chat_runner.py` with injected fake client.
- `tests/unit/test_changed_plugins.py` — Unit tests for `_changed_plugins.py` with monkeypatched `subprocess`.
- `tests/unit/conftest.py` — Empty file to mark `tests/unit/` as a pytest collection root (prevents collision with `plugins/` testpath).
- `plugins/codexis/acceptance/e2e/cdxctl-lifecycle.yaml` — First real end-to-end test (smoke).
- `Makefile` — `test-e2e`, `test-e2e-changed`, `test-e2e-only` targets.
- `.gitlab-ci.yml` — New `plugin-e2e` job (if file doesn't exist yet, create; if it exists, add the job — check first).

**Modified files:**
- `tests/test-marketplace.py` — Replace inline `DaemonClient` class with `from _daemon_client import DaemonClient`.
- `pyproject.toml` — Add `[project]` block with `optional-dependencies.dev = ["pytest", "pyyaml", "jsonpath-ng"]` and widen `testpaths` to include `tests/`.

---

## Task 1: Add dev dependencies and widen pytest testpaths

**Files:**
- Modify: `pyproject.toml`

- [ ] **Step 1: Update `pyproject.toml`**

Replace the entire content of `pyproject.toml` with:

```toml
[project]
name = "codexis-marketplace"
version = "0.0.0"
requires-python = ">=3.11"

[project.optional-dependencies]
dev = [
    "pytest>=7",
    "pyyaml>=6",
    "jsonpath-ng>=1.6",
]

[tool.setuptools]
py-modules = []

[tool.pytest.ini_options]
testpaths = ["tests", "plugins"]
python_files = ["test_*.py", "test-*.py"]
addopts = "-ra --import-mode=importlib"
```

Why `test-*.py`: the existing `test-marketplace.py` uses a hyphen. We keep that pattern; pytest supports multiple globs.

Why `[tool.setuptools] py-modules = []`: without it, setuptools auto-discovers both `plugins/` and `frontend/` as Python packages and errors out with "multiple top-level packages discovered". We're not distributing any Python code from this repo — this is a metadata-only project for the pytest suite — so we tell setuptools there are no modules to install.

- [ ] **Step 2: Install dev deps**

Run: `pip install -e '.[dev]'`
Expected: no errors; pytest, PyYAML, jsonpath-ng available.

- [ ] **Step 3: Verify nothing broke**

Run: `pytest --collect-only -q`
Expected: existing collection still works (no errors). `test-marketplace.py` may or may not be collected (it uses argparse and exits without args) — either is fine as long as there are no import errors.

- [ ] **Step 4: Commit**

```bash
git add pyproject.toml
git commit -m "test: add pytest dev deps and widen testpaths for e2e suite"
```

---

## Task 2: Extract `DaemonClient` into a shared module

**Files:**
- Create: `tests/_daemon_client.py`
- Modify: `tests/test-marketplace.py`

- [ ] **Step 1: Create `tests/_daemon_client.py`**

Copy the `DaemonClient` class and all `GraphQL queries` constants (lines 82–216) from `tests/test-marketplace.py` into a new file `tests/_daemon_client.py`. Add a module docstring:

```python
"""GraphQL client for cdx-daemon. Shared by acceptance and e2e test suites."""

from __future__ import annotations

import base64
import json
import urllib.request
from typing import Any


def encode_node_id(type_name: str, identifier: str) -> str:
    return base64.b64encode(f"{type_name}:{identifier}".encode()).decode()


class DaemonClient:
    # ... copy the class body verbatim from test-marketplace.py
    ...


# GraphQL queries (copy the module-level constants verbatim)
ADD_MARKETPLACE = """..."""
REMOVE_MARKETPLACE = """..."""
LIST_MARKETPLACES = """..."""
INSTALL_PLUGIN = """..."""
UNINSTALL_PLUGIN = """..."""
SKILLS = """..."""
GET_ENTRY = """..."""
```

Keep every string and method body byte-identical to the original to avoid regressions.

- [ ] **Step 2: Update `tests/test-marketplace.py` to import from the new module**

Remove the `DaemonClient` class, the `encode_node_id` function, and the seven GraphQL query constants from `tests/test-marketplace.py`. In their place, add (immediately after the existing top-of-file imports):

```python
sys.path.insert(0, str(Path(__file__).resolve().parent))

from _daemon_client import (
    DaemonClient,
    encode_node_id,
    ADD_MARKETPLACE,
    REMOVE_MARKETPLACE,
    LIST_MARKETPLACES,
    INSTALL_PLUGIN,
    UNINSTALL_PLUGIN,
    SKILLS,
    GET_ENTRY,
)
```

The `sys.path.insert` line is necessary: when pytest collects `test-marketplace.py` (it matches the `test-*.py` glob), it imports the file but does NOT put the test file's directory on `sys.path`. Without the insertion the bare `from _daemon_client import ...` raises `ModuleNotFoundError`. When the script is invoked directly (`python3 tests/test-marketplace.py …`) the line is a no-op because the directory is already on `sys.path[0]`. Same pattern is used in Task 11.

- [ ] **Step 3: Sanity-check the refactor**

Run: `python3 -c "import sys; sys.path.insert(0, 'tests'); from _daemon_client import DaemonClient, ADD_MARKETPLACE; import importlib.util; spec = importlib.util.spec_from_file_location('tm', 'tests/test-marketplace.py'); m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m); print('imports ok')"`
Expected: prints `imports ok`. No `ImportError`, no `NameError`.

- [ ] **Step 4: Verify argparse still works**

Run: `python3 tests/test-marketplace.py --help`
Expected: argparse help output showing `--daemon`, `--token`, `--git-url`, `--git-ref`. Exit code 0.

- [ ] **Step 5: Commit**

```bash
git add tests/_daemon_client.py tests/test-marketplace.py
git commit -m "refactor(tests): extract DaemonClient into shared module"
```

---

## Task 3: Implement `_changed_plugins.py` — changed-plugin detector (TDD)

**Files:**
- Create: `tests/unit/conftest.py`
- Create: `tests/unit/test_changed_plugins.py`
- Create: `tests/_changed_plugins.py`

- [ ] **Step 1: Create `tests/unit/conftest.py`**

```python
# Marker file: makes tests/unit/ a collection root for unit tests.
# Keeps them separate from the integration tests in tests/test-*.py
# (which need --daemon / --token CLI args).
```

- [ ] **Step 2: Write failing tests**

Create `tests/unit/test_changed_plugins.py`:

```python
from pathlib import Path
from unittest.mock import patch
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from _changed_plugins import get_changed_plugins, CORE_TRIGGERS


class FakeRepo:
    """Represents a repo with certain plugin directories existing."""
    def __init__(self, tmp_path: Path, plugins: list[str]):
        self.root = tmp_path
        (tmp_path / ".claude-plugin").mkdir()
        (tmp_path / ".claude-plugin" / "marketplace.json").write_text("{}")
        for p in plugins:
            d = tmp_path / "plugins" / p / "acceptance" / "e2e"
            d.mkdir(parents=True)
            (d / "foo.yaml").write_text("name: foo\nsteps: []\n")


def test_detects_single_changed_plugin(tmp_path):
    FakeRepo(tmp_path, ["codexis", "katastr"])
    with patch("_changed_plugins._git_diff_files",
               return_value=["plugins/codexis/skills/codexis/SKILL.md"]):
        assert get_changed_plugins("main", tmp_path) == ["codexis"]


def test_detects_multiple_changed_plugins(tmp_path):
    FakeRepo(tmp_path, ["codexis", "katastr", "ocr"])
    with patch("_changed_plugins._git_diff_files",
               return_value=[
                   "plugins/codexis/SKILL.md",
                   "plugins/ocr/bin/x",
                   "README.md",
               ]):
        assert get_changed_plugins("main", tmp_path) == ["codexis", "ocr"]


def test_core_trigger_runs_all_plugins_with_e2e(tmp_path):
    FakeRepo(tmp_path, ["codexis", "katastr"])
    # Plugin without acceptance/e2e should NOT be in "all" set.
    (tmp_path / "plugins" / "bare").mkdir(parents=True)
    with patch("_changed_plugins._git_diff_files",
               return_value=[".claude-plugin/marketplace.json"]):
        assert get_changed_plugins("main", tmp_path) == ["codexis", "katastr"]


def test_deleted_plugin_dir_is_ignored(tmp_path):
    FakeRepo(tmp_path, ["codexis"])
    with patch("_changed_plugins._git_diff_files",
               return_value=["plugins/removed-plugin/SKILL.md"]):
        assert get_changed_plugins("main", tmp_path) == []


def test_core_triggers_list_is_explicit():
    # Guard against accidental wildcarding of core-trigger detection.
    assert ".claude-plugin/marketplace.json" in CORE_TRIGGERS
    assert "tests/test-plugin-e2e.py" in CORE_TRIGGERS
    assert "pyproject.toml" in CORE_TRIGGERS


def test_empty_diff_returns_empty(tmp_path):
    FakeRepo(tmp_path, ["codexis"])
    with patch("_changed_plugins._git_diff_files", return_value=[]):
        assert get_changed_plugins("main", tmp_path) == []
```

- [ ] **Step 3: Run tests — expect failure**

Run: `pytest tests/unit/test_changed_plugins.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named '_changed_plugins'`.

- [ ] **Step 4: Implement `tests/_changed_plugins.py`**

```python
"""Detect which plugins changed between two git refs."""

from __future__ import annotations

import subprocess
from pathlib import Path


CORE_TRIGGERS = frozenset([
    ".claude-plugin/marketplace.json",
    "tests/test-plugin-e2e.py",
    "tests/_daemon_client.py",
    "tests/_chat_runner.py",
    "tests/_assertions.py",
    "tests/_changed_plugins.py",
    "tests/_transcript.py",
    "pyproject.toml",
])


def _git_diff_files(base_ref: str, repo_root: Path) -> list[str]:
    result = subprocess.run(
        ["git", "diff", "--name-only", f"{base_ref}...HEAD"],
        cwd=str(repo_root),
        capture_output=True,
        text=True,
        check=True,
    )
    return [line for line in result.stdout.splitlines() if line]


def _all_plugins_with_e2e(repo_root: Path) -> list[str]:
    plugins_dir = repo_root / "plugins"
    if not plugins_dir.is_dir():
        return []
    return sorted(
        p.name for p in plugins_dir.iterdir()
        if p.is_dir() and (p / "acceptance" / "e2e").is_dir()
    )


def get_changed_plugins(base_ref: str, repo_root: Path) -> list[str]:
    """Return sorted list of plugin names affected by diff.

    Returns all plugins with acceptance/e2e/ if any CORE_TRIGGERS file changed.
    """
    files = _git_diff_files(base_ref, repo_root)
    plugins_dir = repo_root / "plugins"
    changed: set[str] = set()
    for path in files:
        if path in CORE_TRIGGERS:
            return _all_plugins_with_e2e(repo_root)
        if path.startswith("plugins/"):
            parts = path.split("/")
            if len(parts) >= 2:
                name = parts[1]
                if (plugins_dir / name).is_dir():
                    changed.add(name)
    return sorted(changed)
```

- [ ] **Step 5: Run tests — expect pass**

Run: `pytest tests/unit/test_changed_plugins.py -v`
Expected: all 6 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tests/_changed_plugins.py tests/unit/test_changed_plugins.py tests/unit/conftest.py
git commit -m "test: add changed-plugins detector with unit tests"
```

---

## Task 4: Implement `_chat_runner.py` — parsing assistant messages (TDD)

**Files:**
- Create: `tests/unit/test_chat_runner.py`
- Create: `tests/_chat_runner.py`

- [ ] **Step 1: Write failing tests for `parse_assistant_message`**

Create `tests/unit/test_chat_runner.py`:

```python
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
```

- [ ] **Step 2: Run tests — expect failure**

Run: `pytest tests/unit/test_chat_runner.py -v`
Expected: FAIL with `ModuleNotFoundError`.

- [ ] **Step 3: Implement `parse_assistant_message`**

Create `tests/_chat_runner.py` (partial; the rest is added in Task 5):

```python
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
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pytest tests/unit/test_chat_runner.py -v`
Expected: all 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/_chat_runner.py tests/unit/test_chat_runner.py
git commit -m "test: add assistant-message parser with unit tests"
```

---

## Task 5: Add `ChatRunner` class that drives the daemon (TDD with injected client)

**Files:**
- Modify: `tests/_chat_runner.py`
- Modify: `tests/unit/test_chat_runner.py`

- [ ] **Step 1: Extend the failing tests**

Append to `tests/unit/test_chat_runner.py`:

```python
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
```

- [ ] **Step 2: Run tests — expect failure**

Run: `pytest tests/unit/test_chat_runner.py -v`
Expected: 5 parse tests still PASS, new 4 tests FAIL with `ImportError: cannot import name 'ChatRunner'`.

- [ ] **Step 3: Implement `ChatRunner` and `ChatError`**

Append to `tests/_chat_runner.py`:

```python
import time
from dataclasses import dataclass


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
```

Also add the three GraphQL methods to `DaemonClient` (in `tests/_daemon_client.py`) so `ChatRunner` can use the real client. Append to the `DaemonClient` class:

```python
    def new_chat(self, model: str | None = None) -> dict:
        data = self.gql_data(
            "mutation NewChat($model: ChatModel) { newChat(model: $model) { id chatId status } }",
            {"model": model},
        )
        return data["newChat"]

    def send_message(self, chat_id: str, message: str) -> dict:
        data = self.gql_data(
            """mutation SendMessage($chatId: ID!, $message: String!) {
                 sendMessage(chatId: $chatId, message: $message) {
                   chatId
                   ... on SendMessageProcessing { executionId }
                   ... on SendMessageError { message }
                 }
               }""",
            {"chatId": chat_id, "message": message},
        )
        return data["sendMessage"]

    def get_chat(self, node_id: str) -> dict:
        data = self.gql_data(
            """query GetChat($id: ID!) {
                 node(id: $id) {
                   ... on ChatInfo {
                     id chatId status
                     messages {
                       __typename id status
                       parts {
                         __typename partId
                         ... on TextMessagePart { content }
                         ... on ThinkingMessagePart { toolCount toolChainId thinkingState: state }
                         ... on ToolMessagePart { toolCallId toolName input output }
                       }
                     }
                   }
                 }
               }""",
            {"id": node_id},
        )
        return data["node"] or {}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pytest tests/unit/test_chat_runner.py -v`
Expected: all 9 tests PASS (5 parse + 4 runner).

- [ ] **Step 5: Commit**

```bash
git add tests/_chat_runner.py tests/unit/test_chat_runner.py tests/_daemon_client.py
git commit -m "feat(tests): add ChatRunner and chat GraphQL methods"
```

---

## Task 6: Implement `_assertions.py` — variable substitution and `input_contains` (TDD)

**Files:**
- Create: `tests/unit/test_assertions.py`
- Create: `tests/_assertions.py`

- [ ] **Step 1: Write failing tests for substitution and `input_contains`**

Create `tests/unit/test_assertions.py`:

```python
from pathlib import Path
import re
import sys
import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from _assertions import substitute, matches_subset


# --- substitute ---------------------------------------------------

def test_substitute_replaces_single_var():
    assert substitute("id={{ x }}", {"x": "abc"}) == "id=abc"


def test_substitute_handles_no_vars():
    assert substitute("no vars here", {"x": "abc"}) == "no vars here"


def test_substitute_json_encodes_non_strings():
    assert substitute("v={{ x }}", {"x": 42}) == "v=42"


def test_substitute_raises_on_missing_var():
    with pytest.raises(KeyError, match="missing"):
        substitute("hi {{ missing }}", {})


# --- matches_subset ----------------------------------------------

def test_subset_literal_match_passes():
    assert matches_subset({"a": 1, "b": 2}, {"a": 1, "b": 2, "c": 3}) is None


def test_subset_missing_key_fails():
    err = matches_subset({"a": 1}, {"b": 1})
    assert err is not None and "missing key 'a'" in err


def test_subset_value_mismatch_fails():
    err = matches_subset({"a": 1}, {"a": 2})
    assert err is not None and "value mismatch" in err


def test_subset_regex_prefix_passes():
    assert matches_subset({"name": "~/^e2e-/"}, {"name": "e2e-foo"}) is None


def test_subset_regex_prefix_fails():
    err = matches_subset({"name": "~/^e2e-/"}, {"name": "prod-foo"})
    assert err is not None and "regex" in err


def test_subset_nested_dict_recurses():
    assert matches_subset({"a": {"b": 1}}, {"a": {"b": 1, "c": 2}}) is None
    err = matches_subset({"a": {"b": 1}}, {"a": {"b": 2}})
    assert err is not None
```

- [ ] **Step 2: Run tests — expect failure**

Run: `pytest tests/unit/test_assertions.py -v`
Expected: FAIL with `ModuleNotFoundError`.

- [ ] **Step 3: Implement substitution and subset matching**

Create `tests/_assertions.py`:

```python
"""YAML assertion vocabulary interpreter.

Pure functions. No I/O. Raise AssertionError (or return error-string, where noted)
on failures; caller formats the error message.
"""

from __future__ import annotations

import json
import re
from typing import Any


_VAR_PATTERN = re.compile(r"\{\{\s*(\w+)\s*\}\}")


def substitute(template: str, vars_: dict[str, Any]) -> str:
    """Replace {{ name }} with vars_[name]. Non-string values are JSON-encoded.

    Raises KeyError if a referenced variable is missing.
    """
    def _repl(m: re.Match) -> str:
        key = m.group(1)
        if key not in vars_:
            raise KeyError(f"missing variable: {key!r}")
        val = vars_[key]
        if isinstance(val, str):
            return val
        return json.dumps(val)
    return _VAR_PATTERN.sub(_repl, template)


def _is_regex_marker(v: Any) -> bool:
    return isinstance(v, str) and v.startswith("~/") and v.endswith("/") and len(v) >= 4


def _extract_regex(v: str) -> str:
    return v[2:-1]


def matches_subset(expected: Any, actual: Any, path: str = "") -> str | None:
    """Check that `expected` is a recursive subset of `actual`.

    Returns None on match, or a human-readable error message on mismatch.
    `expected` can contain regex markers (`~/pattern/`) at leaf positions.
    """
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            return f"at {path or '<root>'}: expected dict, got {type(actual).__name__}"
        for k, v in expected.items():
            sub_path = f"{path}.{k}" if path else k
            if k not in actual:
                return f"at {sub_path}: missing key {k!r} in actual"
            err = matches_subset(v, actual[k], sub_path)
            if err:
                return err
        return None
    if isinstance(expected, list):
        if not isinstance(actual, list) or len(actual) < len(expected):
            return f"at {path or '<root>'}: expected list of >= {len(expected)}, got {actual!r}"
        for i, (e, a) in enumerate(zip(expected, actual)):
            err = matches_subset(e, a, f"{path}[{i}]")
            if err:
                return err
        return None
    if _is_regex_marker(expected):
        pattern = _extract_regex(expected)
        if not isinstance(actual, str):
            return f"at {path or '<root>'}: regex {pattern!r} needs string, got {type(actual).__name__}"
        if re.search(pattern, actual):
            return None
        return f"at {path or '<root>'}: regex {pattern!r} did not match {actual!r}"
    if expected == actual:
        return None
    return f"at {path or '<root>'}: value mismatch — expected {expected!r}, got {actual!r}"
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pytest tests/unit/test_assertions.py -v`
Expected: all 9 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/_assertions.py tests/unit/test_assertions.py
git commit -m "test: add substitute() and matches_subset() primitives"
```

---

## Task 7: Implement `_assertions.py` — step-level output and tool-call assertions (TDD)

**Files:**
- Modify: `tests/_assertions.py`
- Modify: `tests/unit/test_assertions.py`

- [ ] **Step 1: Extend failing tests**

Append to `tests/unit/test_assertions.py`:

```python
from _assertions import run_step_assertions, AssertionFailure


def _result(text="", tool_calls=()):
    return {"text": text, "tool_calls": list(tool_calls)}


# --- output_* ---------------------------------------------------

def test_output_contains_passes():
    run_step_assertions({"output_contains": "hello"}, _result(text="hello world"), {})


def test_output_contains_fails():
    with pytest.raises(AssertionFailure, match="output_contains"):
        run_step_assertions({"output_contains": "missing"}, _result(text="hi"), {})


def test_output_not_contains_passes():
    run_step_assertions({"output_not_contains": "error"}, _result(text="all good"), {})


def test_output_not_contains_fails():
    with pytest.raises(AssertionFailure, match="output_not_contains"):
        run_step_assertions({"output_not_contains": "bad"}, _result(text="bad news"), {})


def test_output_matches_regex_passes():
    run_step_assertions({"output_matches": r"id=\d+"}, _result(text="created id=42"), {})


def test_output_matches_regex_fails():
    with pytest.raises(AssertionFailure, match="output_matches"):
        run_step_assertions({"output_matches": r"^\d+$"}, _result(text="nope"), {})


def test_output_substitution_with_captured_var():
    run_step_assertions(
        {"output_contains": "{{ id }}"}, _result(text="got auto_7"), {"id": "auto_7"},
    )


# --- tool_calls -------------------------------------------------

def test_tool_calls_passes_on_match():
    run_step_assertions(
        {"tool_calls": [{"name": "cdxctl", "input_contains": {"subcommand": "list"}}]},
        _result(tool_calls=[{"name": "cdxctl",
                             "input": {"subcommand": "list", "verbose": True},
                             "output": {"items": []}}]),
        {},
    )


def test_tool_calls_fails_on_missing_call():
    with pytest.raises(AssertionFailure, match="no matching call"):
        run_step_assertions(
            {"tool_calls": [{"name": "cdxctl"}]},
            _result(tool_calls=[{"name": "other", "input": {}, "output": {}}]),
            {},
        )


def test_tool_calls_enforces_order():
    """Second expected must match an actual call after the one matched by the first."""
    with pytest.raises(AssertionFailure):
        run_step_assertions(
            {"tool_calls": [
                {"name": "cdxctl", "input_contains": {"subcommand": "list"}},
                {"name": "cdxctl", "input_contains": {"subcommand": "create"}},
            ]},
            _result(tool_calls=[
                {"name": "cdxctl", "input": {"subcommand": "create"}, "output": {}},
                {"name": "cdxctl", "input": {"subcommand": "list"}, "output": {}},
            ]),
            {},
        )


def test_tool_calls_empty_list_means_no_tools_called():
    run_step_assertions({"tool_calls": []}, _result(tool_calls=[]), {})
    with pytest.raises(AssertionFailure, match="expected no tool calls"):
        run_step_assertions(
            {"tool_calls": []},
            _result(tool_calls=[{"name": "x", "input": {}, "output": {}}]),
            {},
        )


def test_tool_calls_absent_means_no_assertion():
    # No tool_calls key at all = allow any or none
    run_step_assertions({}, _result(tool_calls=[{"name": "x", "input": {}, "output": {}}]), {})
    run_step_assertions({}, _result(tool_calls=[]), {})


def test_tool_calls_substitutes_vars_in_input_contains():
    run_step_assertions(
        {"tool_calls": [{"name": "cdxctl",
                         "input_contains": {"id": "{{ auto_id }}"}}]},
        _result(tool_calls=[{"name": "cdxctl",
                             "input": {"id": "auto_7"}, "output": {}}]),
        {"auto_id": "auto_7"},
    )
```

- [ ] **Step 2: Run tests — expect failure**

Run: `pytest tests/unit/test_assertions.py -v`
Expected: new tests FAIL with `ImportError: cannot import name 'run_step_assertions'`.

- [ ] **Step 3: Implement `run_step_assertions` and `AssertionFailure`**

Append to `tests/_assertions.py`:

```python
class AssertionFailure(AssertionError):
    """Rich test failure raised by run_step_assertions."""


def _substitute_deep(value: Any, vars_: dict) -> Any:
    """Recursively substitute {{var}} in string leaves of a nested structure."""
    if isinstance(value, str):
        return substitute(value, vars_)
    if isinstance(value, dict):
        return {k: _substitute_deep(v, vars_) for k, v in value.items()}
    if isinstance(value, list):
        return [_substitute_deep(v, vars_) for v in value]
    return value


def _assert_output(expect: dict, text: str, vars_: dict) -> None:
    if "output_contains" in expect:
        needle = substitute(expect["output_contains"], vars_)
        if needle not in text:
            raise AssertionFailure(
                f"output_contains failed: {needle!r} not in {text!r}"
            )
    if "output_not_contains" in expect:
        needle = substitute(expect["output_not_contains"], vars_)
        if needle in text:
            raise AssertionFailure(
                f"output_not_contains failed: {needle!r} IS in {text!r}"
            )
    if "output_matches" in expect:
        pattern = substitute(expect["output_matches"], vars_)
        if not re.search(pattern, text):
            raise AssertionFailure(
                f"output_matches failed: /{pattern}/ did not match {text!r}"
            )


def _assert_tool_calls(expected_list: list[dict], actual_calls: list[dict], vars_: dict) -> None:
    # Empty list = strict "no tool calls"
    if not expected_list:
        if actual_calls:
            names = [c.get("name") for c in actual_calls]
            raise AssertionFailure(f"expected no tool calls, got {names}")
        return

    cursor = 0
    for i, expected in enumerate(expected_list):
        want_name = expected.get("name")
        want_input = _substitute_deep(expected.get("input_contains", {}), vars_)
        matched_at = None
        for j in range(cursor, len(actual_calls)):
            call = actual_calls[j]
            if call.get("name") != want_name:
                continue
            err = matches_subset(want_input, call.get("input") or {})
            if err is None:
                matched_at = j
                break
        if matched_at is None:
            raise AssertionFailure(
                f"tool_calls[{i}] ({want_name}): no matching call found "
                f"in actual calls from index {cursor} onward. "
                f"Actual: {actual_calls!r}"
            )
        cursor = matched_at + 1


def run_step_assertions(expect: dict, result: dict, captured: dict) -> None:
    """Run every assertion declared in `expect` against the parsed step `result`."""
    if "tool_calls" in expect:
        _assert_tool_calls(expect["tool_calls"], result["tool_calls"], captured)
    _assert_output(expect, result["text"], captured)
    # capture and judge are handled by the caller (Task 8, Task 9)
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pytest tests/unit/test_assertions.py -v`
Expected: all 22 assertion tests PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/_assertions.py tests/unit/test_assertions.py
git commit -m "test: add run_step_assertions for output and tool_calls"
```

---

## Task 8: Implement `capture` via jsonpath (TDD)

**Files:**
- Modify: `tests/_assertions.py`
- Modify: `tests/unit/test_assertions.py`

- [ ] **Step 1: Extend failing tests**

Append to `tests/unit/test_assertions.py`:

```python
from _assertions import apply_captures


def test_capture_extracts_from_tool_output():
    captured = apply_captures(
        {"aid": "$.tool_calls[0].output.id"},
        _result(tool_calls=[{"name": "x", "input": {}, "output": {"id": "auto_9"}}]),
    )
    assert captured == {"aid": "auto_9"}


def test_capture_multiple_values():
    captured = apply_captures(
        {"aid": "$.tool_calls[0].output.id", "first_text": "$.text"},
        _result(text="hello", tool_calls=[
            {"name": "x", "input": {}, "output": {"id": "auto_9"}},
        ]),
    )
    assert captured == {"aid": "auto_9", "first_text": "hello"}


def test_capture_unresolved_path_raises():
    with pytest.raises(AssertionFailure, match="did not resolve"):
        apply_captures(
            {"aid": "$.tool_calls[5].output.id"},
            _result(tool_calls=[]),
        )


def test_capture_no_captures_returns_empty():
    assert apply_captures(None, _result(text="x")) == {}
    assert apply_captures({}, _result(text="x")) == {}
```

- [ ] **Step 2: Run tests — expect failure**

Run: `pytest tests/unit/test_assertions.py -v`
Expected: FAIL with `ImportError: cannot import name 'apply_captures'`.

- [ ] **Step 3: Implement `apply_captures`**

Append to `tests/_assertions.py`:

```python
from jsonpath_ng.ext import parse as _jp_parse


def apply_captures(captures: dict | None, result: dict) -> dict:
    """Evaluate jsonpath expressions against the step result and return {name: value}.

    Raises AssertionFailure if any jsonpath resolves to nothing.
    """
    if not captures:
        return {}
    out: dict = {}
    for name, path in captures.items():
        try:
            expr = _jp_parse(path)
        except Exception as e:
            raise AssertionFailure(f"capture {name!r}: invalid jsonpath {path!r}: {e}") from e
        matches = [m.value for m in expr.find(result)]
        if not matches:
            raise AssertionFailure(
                f"capture {name!r}: jsonpath {path!r} did not resolve against result"
            )
        out[name] = matches[0]
    return out
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pytest tests/unit/test_assertions.py -v`
Expected: all 26 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/_assertions.py tests/unit/test_assertions.py
git commit -m "test: add apply_captures via jsonpath"
```

---

## Task 9: Add `judge` assertion (LLM-as-judge escape hatch, TDD with fake client)

**Files:**
- Modify: `tests/_assertions.py`
- Modify: `tests/unit/test_assertions.py`

- [ ] **Step 1: Write failing tests**

Append to `tests/unit/test_assertions.py`:

```python
from _assertions import run_judge


class FakeJudgeClient:
    def __init__(self, scripted_reply: str):
        self.scripted_reply = scripted_reply
        self.last_prompt: str | None = None

    def run_single_shot_chat(self, prompt: str) -> str:
        self.last_prompt = prompt
        return self.scripted_reply


def test_judge_passes():
    fake = FakeJudgeClient('{"pass": true, "reason": "matches rubric"}')
    run_judge(
        judge_spec={"rubric": "response must mention zákon"},
        result=_result(text="zákon je..."),
        client=fake,
    )
    assert "zákon" in (fake.last_prompt or "")


def test_judge_fails_with_reason():
    fake = FakeJudgeClient('{"pass": false, "reason": "missing citation"}')
    with pytest.raises(AssertionFailure, match="missing citation"):
        run_judge(
            judge_spec={"rubric": "response must cite Art. 123"},
            result=_result(text="no citation here"),
            client=fake,
        )


def test_judge_rejects_malformed_reply():
    fake = FakeJudgeClient("not json")
    with pytest.raises(AssertionFailure, match="judge reply"):
        run_judge(
            judge_spec={"rubric": "whatever"},
            result=_result(text="x"),
            client=fake,
        )
```

- [ ] **Step 2: Run tests — expect failure**

Run: `pytest tests/unit/test_assertions.py -v`
Expected: FAIL with `ImportError: cannot import name 'run_judge'`.

- [ ] **Step 3: Implement `run_judge`**

Append to `tests/_assertions.py`:

```python
_JUDGE_TEMPLATE = """You are grading whether an AI assistant's response satisfies a rubric.

RUBRIC:
{rubric}

ASSISTANT'S FINAL TEXT:
{text}

TOOL CALLS MADE DURING THIS TURN (JSON):
{tool_calls_json}

Reply with ONLY a compact JSON object of the form:
{{"pass": <true|false>, "reason": "<one sentence>"}}
"""


def run_judge(judge_spec: dict, result: dict, client: Any) -> None:
    """Invoke an LLM judge via `client.run_single_shot_chat(prompt) -> str`.

    Raises AssertionFailure if the judge returns pass=false or malformed JSON.
    """
    rubric = judge_spec.get("rubric") or ""
    prompt = _JUDGE_TEMPLATE.format(
        rubric=rubric,
        text=result.get("text", ""),
        tool_calls_json=json.dumps(result.get("tool_calls", []), ensure_ascii=False),
    )
    raw = client.run_single_shot_chat(prompt)
    try:
        verdict = json.loads(raw)
    except json.JSONDecodeError as e:
        raise AssertionFailure(f"judge reply was not JSON: {raw!r}") from e
    if not verdict.get("pass"):
        reason = verdict.get("reason") or "(no reason given)"
        raise AssertionFailure(f"judge rejected: {reason}")
```

Also add a helper method to `DaemonClient` (in `tests/_daemon_client.py`) so real judge runs work:

```python
    def run_single_shot_chat(self, prompt: str, model: str | None = None,
                             poll_interval_s: float = 2.0,
                             poll_timeout_s: float = 120.0) -> str:
        """Start a fresh chat, send one prompt, return the assistant's final text."""
        import time
        info = self.new_chat(model)
        self.send_message(info["chatId"], prompt)
        deadline = time.monotonic() + poll_timeout_s
        while True:
            chat = self.get_chat(info["id"])
            status = chat.get("status")
            if status == "ERROR":
                raise RuntimeError(f"judge chat ERROR: {chat}")
            if status == "READY" and chat.get("messages"):
                msg = chat["messages"][-1]
                parts = msg.get("parts") or []
                return "".join(
                    p.get("content") or "" for p in parts
                    if p.get("__typename") == "TextMessagePart"
                )
            if time.monotonic() >= deadline:
                raise RuntimeError(f"judge chat timed out after {poll_timeout_s}s")
            time.sleep(poll_interval_s)
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pytest tests/unit/test_assertions.py -v`
Expected: all 29 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/_assertions.py tests/unit/test_assertions.py tests/_daemon_client.py
git commit -m "test: add LLM-as-judge assertion"
```

---

## Task 10: Implement `_transcript.py` — Markdown transcript writer (TDD)

**Files:**
- Create: `tests/unit/test_transcript.py`
- Create: `tests/_transcript.py`

- [ ] **Step 1: Write failing tests**

Create `tests/unit/test_transcript.py`:

```python
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
```

- [ ] **Step 2: Run tests — expect failure**

Run: `pytest tests/unit/test_transcript.py -v`
Expected: FAIL with `ModuleNotFoundError`.

- [ ] **Step 3: Implement `render_transcript` and `write_transcript`**

Create `tests/_transcript.py`:

```python
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
```

- [ ] **Step 4: Run tests — expect pass**

Run: `pytest tests/unit/test_transcript.py -v`
Expected: all 3 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/_transcript.py tests/unit/test_transcript.py
git commit -m "test: add transcript renderer"
```

---

## Task 11: Implement `test-plugin-e2e.py` — pytest collection + per-plugin fixtures

**Files:**
- Create: `tests/test-plugin-e2e.py`

- [ ] **Step 1: Create the pytest entry point**

Create `tests/test-plugin-e2e.py`:

```python
"""Plugin E2E test suite.

Discovers plugins/<name>/acceptance/e2e/*.yaml files and runs each as a
multi-turn chat test against a live cdx-daemon.

Invocation (CI):
  pytest tests/test-plugin-e2e.py \
    --daemon URL --token JWT \
    --git-url URL --git-ref REF --base-ref REF \
    --changed

Invocation (local full run):
  pytest tests/test-plugin-e2e.py \
    --daemon URL --token JWT --git-url URL --git-ref REF
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest
import yaml

# Make sibling modules importable (--import-mode=importlib means no packages).
sys.path.insert(0, str(Path(__file__).resolve().parent))

from _assertions import AssertionFailure, apply_captures, run_step_assertions, substitute
from _chat_runner import ChatRunner
from _changed_plugins import get_changed_plugins
from _daemon_client import DaemonClient, encode_node_id
from _transcript import write_transcript


# -------------------------------------------------------------- CLI options

def pytest_addoption(parser):
    g = parser.getgroup("plugin-e2e")
    g.addoption("--daemon", action="store", help="cdx-daemon base URL")
    g.addoption("--token", action="store", help="JWT bearer token")
    g.addoption("--git-url", action="store", help="Marketplace git URL")
    g.addoption("--git-ref", action="store", help="Marketplace git ref (merge-result)")
    g.addoption("--base-ref", action="store", default="origin/main",
                help="Base ref for change detection (default: origin/main)")
    g.addoption("--changed", action="store_true",
                help="Run only plugins changed between --base-ref and HEAD")
    g.addoption("--only", action="store", default="",
                help="Comma-separated list of plugin names; overrides --changed")
    g.addoption("--transcript-dir", action="store",
                default="test-results/transcripts",
                help="Where to write failure transcripts")


# -------------------------------------------------------------- Collection

MARKETPLACE_ROOT = Path(__file__).resolve().parent.parent


def _plugins_to_test(config) -> list[str]:
    only = (config.getoption("--only") or "").strip()
    if only:
        return sorted({n.strip() for n in only.split(",") if n.strip()})
    if config.getoption("--changed"):
        base = config.getoption("--base-ref") or "origin/main"
        return get_changed_plugins(base, MARKETPLACE_ROOT)
    # Default: every plugin that has acceptance/e2e/
    return sorted(
        p.name for p in (MARKETPLACE_ROOT / "plugins").iterdir()
        if p.is_dir() and (p / "acceptance" / "e2e").is_dir()
    )


def _yaml_files_for(plugin: str) -> list[Path]:
    d = MARKETPLACE_ROOT / "plugins" / plugin / "acceptance" / "e2e"
    if not d.is_dir():
        return []
    return sorted(d.glob("*.yaml"))


def pytest_generate_tests(metafunc):
    if "yaml_path" not in metafunc.fixturenames:
        return
    cases: list[tuple[str, Path]] = []
    for plugin in _plugins_to_test(metafunc.config):
        for yml in _yaml_files_for(plugin):
            cases.append((plugin, yml))
    metafunc.parametrize(
        ("plugin_name", "yaml_path"), cases,
        ids=[f"{p}/{y.stem}" for p, y in cases],
    )


# -------------------------------------------------------------- Fixtures

@pytest.fixture(scope="session")
def daemon_client(request) -> DaemonClient:
    url = request.config.getoption("--daemon") or os.environ.get("DAEMON_URL")
    tok = request.config.getoption("--token") or os.environ.get("SERVICE_JWT")
    if not url or not tok:
        pytest.skip("--daemon and --token required")
    c = DaemonClient(url, tok)
    if not c.health_check():
        pytest.skip("daemon not healthy")
    return c


@pytest.fixture(scope="session")
def marketplace(request, daemon_client) -> dict:
    """Add the marketplace from the given git ref, tear down at session end."""
    git_url = request.config.getoption("--git-url")
    git_ref = request.config.getoption("--git-ref")
    if not git_url or not git_ref:
        pytest.skip("--git-url and --git-ref required")

    from _daemon_client import ADD_MARKETPLACE, LIST_MARKETPLACES, REMOVE_MARKETPLACE
    manifest = yaml.safe_load((MARKETPLACE_ROOT / ".claude-plugin" / "marketplace.json").read_text())
    mkt_name = manifest["name"]
    mkt_uuid = manifest.get("uuid", "")

    # Remove any stale copy from a previous run.
    try:
        daemon_client.gql_data(REMOVE_MARKETPLACE,
                               {"id": encode_node_id("Marketplace", mkt_uuid)})
    except Exception:
        pass

    data = daemon_client.gql_data(
        ADD_MARKETPLACE,
        {"input": {"sourceType": "GIT", "gitUrl": git_url, "gitRef": git_ref}},
    )
    our = next((m for m in data["addMarketplace"] if m["name"] == mkt_name), None)
    assert our is not None, f"Marketplace {mkt_name!r} not found in addMarketplace reply"

    yield our

    try:
        daemon_client.gql_data(REMOVE_MARKETPLACE, {"id": our["id"]})
    except Exception:
        pass


@pytest.fixture(scope="module")
def _installed_plugins_cache():
    # Cross-test cache so we install each plugin exactly once per session.
    return {"installed": set()}


@pytest.fixture
def installed_plugin(request, daemon_client, marketplace, plugin_name,
                     _installed_plugins_cache):
    """Install `plugin_name` if not already, uninstall at session end."""
    from _daemon_client import INSTALL_PLUGIN, UNINSTALL_PLUGIN

    plugin = next((p for p in marketplace["plugins"] if p["name"] == plugin_name), None)
    if plugin is None:
        pytest.fail(f"plugin {plugin_name!r} not found in marketplace")

    if plugin["id"] not in _installed_plugins_cache["installed"]:
        daemon_client.gql_data(INSTALL_PLUGIN, {"input": {"id": plugin["id"]}})
        _installed_plugins_cache["installed"].add(plugin["id"])

        # Register session-end uninstall via finalizer.
        def _uninstall():
            try:
                daemon_client.gql_data(UNINSTALL_PLUGIN, {"input": {"id": plugin["id"]}})
            except Exception:
                pass
        request.session.addfinalizer(_uninstall)

    return plugin


# -------------------------------------------------------------- Test

def test_plugin_e2e(request, daemon_client, installed_plugin,
                    plugin_name, yaml_path):
    spec = yaml.safe_load(yaml_path.read_text())
    if spec.get("skip"):
        pytest.skip(f"skip: {spec['skip']}")

    test_name = spec.get("name") or yaml_path.stem
    steps = spec.get("steps") or []
    assert steps, f"{yaml_path}: no steps defined"

    runner = ChatRunner(
        daemon_client,
        poll_interval_s=float(os.environ.get("CDX_E2E_POLL_INTERVAL_S", "2")),
        poll_timeout_s=float(os.environ.get("CDX_E2E_POLL_TIMEOUT_S", "600")),
    )
    runner.start()

    import secrets
    captured: dict = {"run_id": secrets.token_hex(2)}
    recorded: list[dict] = []

    try:
        for step in steps:
            prompt = substitute(step["prompt"], captured)
            result = runner.step(prompt)
            expect = step.get("expect") or {}
            try:
                run_step_assertions(expect, result, captured)
                captured.update(apply_captures(expect.get("capture"), result))
                recorded.append({"prompt": prompt, "result": result, "status": "PASS"})
            except (AssertionFailure, AssertionError) as e:
                recorded.append({"prompt": prompt, "result": result,
                                 "status": "FAIL", "error": str(e)})
                raise
    except BaseException:
        out_dir = Path(request.config.getoption("--transcript-dir"))
        if not out_dir.is_absolute():
            out_dir = MARKETPLACE_ROOT / out_dir
        write_transcript(out_dir, plugin_name, test_name, recorded)
        raise
```

- [ ] **Step 2: Sanity-check collection with zero YAMLs**

`pytest_addoption` must live in a `conftest.py` — it is not recognized when declared in a collected test module. Put the option registration in a new `tests/conftest.py` alongside `test-plugin-e2e.py`; the rest of the file (fixtures, `pytest_generate_tests`, the `test_plugin_e2e` function) stays in `test-plugin-e2e.py`.

Run:
```bash
pytest tests/test-plugin-e2e.py --collect-only -q --only=nonexistent
```
Expected behaviour: pytest collects one synthetic `test_plugin_e2e[NOTSET]` item (pytest 8+ behaviour for an empty `parametrize`) which is reported as SKIPPED with message "got empty parameter set". Exit code 0. No import errors.

- [ ] **Step 3: Check `--help` shows the new options**

Run:
```bash
pytest tests/test-plugin-e2e.py --help | grep -E '\-\-(daemon|token|git-url|git-ref|base-ref|changed|only|transcript)'
```
Expected: all 7 options listed.

- [ ] **Step 4: Commit**

```bash
git add tests/test-plugin-e2e.py
git commit -m "feat(tests): add pytest-plugin-e2e collection and fixtures"
```

---

## Task 12: Add Makefile targets

**Files:**
- Create: `Makefile`

- [ ] **Step 1: Create `Makefile`**

```makefile
.PHONY: test-e2e test-e2e-changed test-e2e-only

# Full run — every plugin with acceptance/e2e/
test-e2e:
	pytest tests/test-plugin-e2e.py \
	  --daemon   "$${DAEMON_URL:?set DAEMON_URL}" \
	  --token    "$${SERVICE_JWT:?set SERVICE_JWT}" \
	  --git-url  "$${TEST_GIT_URL:?set TEST_GIT_URL to a remote the daemon can clone}" \
	  --git-ref  "$${TEST_GIT_REF:?set TEST_GIT_REF to the branch you just pushed}" \
	  --base-ref "$${TEST_BASE_REF:-origin/main}" \
	  -v $(PYTEST_EXTRA)

# Only plugins changed vs. --base-ref
test-e2e-changed:
	$(MAKE) test-e2e PYTEST_EXTRA="--changed $(PYTEST_EXTRA)"

# Only a specific plugin: make test-e2e-only PLUGIN=codexis
test-e2e-only:
	@[ -n "$(PLUGIN)" ] || (echo "usage: make test-e2e-only PLUGIN=<name>"; exit 2)
	$(MAKE) test-e2e PYTEST_EXTRA="--only $(PLUGIN) $(PYTEST_EXTRA)"
```

- [ ] **Step 2: Sanity-check targets parse**

Run:
```bash
make -n test-e2e DAEMON_URL=http://x SERVICE_JWT=x TEST_GIT_URL=x TEST_GIT_REF=x 2>&1 | head
```
Expected: prints the pytest command that would run. No "missing variable" errors.

- [ ] **Step 3: Commit**

```bash
git add Makefile
git commit -m "build: add Makefile targets for plugin e2e suite"
```

---

## Task 13: Add GitLab CI job

**Files:**
- Create: `.gitlab-ci.yml` (if it doesn't exist) OR modify existing

- [ ] **Step 1: Check for existing `.gitlab-ci.yml`**

Run: `ls .gitlab-ci.yml 2>/dev/null && cat .gitlab-ci.yml || echo "NO CI FILE"`

If it exists, **read it fully** before editing — the job below must match the existing `image`, `before_script`, `needs`, and daemon-spinup conventions used by `test-marketplace`. If no CI exists yet, create one.

- [ ] **Step 2: Add the `plugin-e2e` job**

Either append this job to the existing `.gitlab-ci.yml`, or (if the file doesn't exist) create it with this content — **adjusting the daemon-spinup dependency to match whatever the repo already uses.** The part below is the new `plugin-e2e` job; stages, services, images, and the `daemon-up` job shape must come from inspection of the existing CI or from the engineer asking the user:

```yaml
plugin-e2e:
  stage: test
  image: python:3.11-slim
  needs:
    - job: daemon-up     # <-- job name must match the repo's existing daemon-spinup job
      artifacts: false
  variables:
    PIP_DISABLE_PIP_VERSION_CHECK: "1"
  before_script:
    - apt-get update && apt-get install -y --no-install-recommends git
    - pip install -e '.[dev]'
    - git fetch origin "$CI_MERGE_REQUEST_TARGET_BRANCH_NAME" --depth=50
  script:
    - pytest tests/test-plugin-e2e.py
        --daemon    "$DAEMON_URL"
        --token     "$SERVICE_JWT"
        --git-url   "$CI_REPOSITORY_URL"
        --git-ref   "$CI_MERGE_REQUEST_REF"
        --base-ref  "origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME"
        --changed
        --junitxml=test-results/plugin-e2e-junit.xml
        -v
  rules:
    - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
  artifacts:
    when: always
    paths:
      - test-results/
    reports:
      junit: test-results/plugin-e2e-junit.xml
```

**Notes for the engineer:**
- `DAEMON_URL` and `SERVICE_JWT` come from GitLab CI/CD variables (already set for `test-marketplace` — reuse them).
- `CI_MERGE_REQUEST_REF` is the merge-result ref populated by GitLab for MR pipelines.
- `git fetch` with depth=50 ensures `origin/main` exists locally so `git diff` works.
- If the repo's existing CI uses a different Python image or different `needs:` structure, align with that — don't introduce a new pattern.

- [ ] **Step 3: Validate YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.gitlab-ci.yml'))"`
Expected: no output, exit code 0.

- [ ] **Step 4: Commit**

```bash
git add .gitlab-ci.yml
git commit -m "ci: add plugin-e2e job"
```

---

## Task 14: Write first real YAML test (codexis cdxctl lifecycle) and smoke-test against a real daemon

**Files:**
- Create: `plugins/codexis/acceptance/e2e/cdxctl-lifecycle.yaml`

This is the *only* task in the plan that requires a running cdx-daemon to verify. Everything up to Task 13 can be fully validated with unit tests and dry runs.

- [ ] **Step 1: Create the YAML**

```yaml
# plugins/codexis/acceptance/e2e/cdxctl-lifecycle.yaml
name: cdxctl-automation-lifecycle
description: Verify cdxctl can create, list, invoke, and delete an automation end-to-end.

steps:
  - prompt: "Using cdxctl, create an automation named e2e-{{ run_id }} that runs the shell command 'echo hello'."
    expect:
      tool_calls:
        - name: cdxctl
          input_contains:
            subcommand: "create"
            name: "~/^e2e-/"
      capture:
        automationId: "$.tool_calls[0].output.id"

  - prompt: "Using cdxctl, list all automations and tell me which ones exist."
    expect:
      tool_calls:
        - name: cdxctl
          input_contains: { subcommand: "list" }
      output_contains: "{{ automationId }}"

  - prompt: "Using cdxctl, invoke the automation with id {{ automationId }}."
    expect:
      tool_calls:
        - name: cdxctl
          input_contains:
            subcommand: "invoke"
            id: "{{ automationId }}"

  - prompt: "Using cdxctl, delete the automation with id {{ automationId }}."
    expect:
      tool_calls:
        - name: cdxctl
          input_contains:
            subcommand: "delete"
            id: "{{ automationId }}"

  - prompt: "Using cdxctl, list automations again and confirm the one I just deleted is gone."
    expect:
      output_not_contains: "{{ automationId }}"
```

**About the exact `subcommand` / input keys:** the spec keys (`subcommand`, `name`, `id`) are our best guess of what the `cdxctl` skill actually exposes. The engineer must run this smoke test once (step 2 below) and iterate on the expected keys until they match what the skill really receives. This YAML is documentation of what *should* work, and iterating on it is part of this task.

- [ ] **Step 2: Smoke-test against a running daemon**

With a locally running daemon and the branch pushed to a remote the daemon can clone:

```bash
export DAEMON_URL=http://localhost:8086
export SERVICE_JWT=<your token>
export TEST_GIT_URL=<this remote, e.g. https://gitlab.agrp.dev/profidata/codexis-marketplace.git>
export TEST_GIT_REF=<your feature branch>
make test-e2e-only PLUGIN=codexis
```

Expected final state: test PASSES. If it fails:
1. Read the transcript written to `test-results/transcripts/codexis/cdxctl-automation-lifecycle.md`.
2. If the model called `cdxctl` but with different input keys than expected → update the YAML's `input_contains` to match what the tool-call trace actually shows. That's the source of truth.
3. If the model did not call `cdxctl` at all → the skill's description needs work (a separate fix outside this plan), or the prompt needs to be more explicit.
4. Re-run until it passes.

Iterate freely on the YAML during step 2 — that's the test authoring loop.

- [ ] **Step 3: Commit the working YAML**

```bash
git add plugins/codexis/acceptance/e2e/cdxctl-lifecycle.yaml
git commit -m "test(codexis): add cdxctl automation lifecycle e2e"
```

---

## Self-review notes

**Spec coverage** — every section of the spec has a task:
- §Architecture → Tasks 2, 4, 5, 11 (client, runner, assertions, collection)
- §YAML test format → Task 14 (real example) + vocabulary implemented in Tasks 6, 7, 8, 9
- §Runner → Tasks 4, 5
- §Change detection → Task 3
- §CI integration → Task 13
- §Local dev → Task 12 (Makefile)
- §Failure artifacts → Task 10, wired in Task 11

**Ordering:** pure-logic modules first (Tasks 3–10) with full TDD and no daemon dependency. Integration piece (Task 11) imports them. CI/Makefile (Tasks 12–13) wrap the integration. First real YAML (Task 14) is last because it's the only thing that needs a real daemon.

**No placeholders:** every step has exact code, exact paths, and exact commands with expected outputs.

**Type consistency:**
- `parse_assistant_message` returns `{text, tool_calls}` — used consistently in ChatRunner, assertions, captures, transcript.
- `tool_calls` element shape `{name, input, output}` — consistent across Task 4, 5, 7, 8, 10.
- `captured` dict name used throughout the runner and assertion layer.

**Known unknowns (and where they're resolved):**
- Exact `cdxctl` input shape — resolved by smoke test in Task 14.
- Exact existing `.gitlab-ci.yml` conventions — resolved by inspection in Task 13 step 1.
