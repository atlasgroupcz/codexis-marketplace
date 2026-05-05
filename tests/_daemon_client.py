"""GraphQL client for cdx-daemon. Shared by acceptance and e2e test suites."""

from __future__ import annotations

import json
import time
import urllib.parse
import urllib.request
import uuid
from typing import Any


# ---------------------------------------------------------------------------
# GraphQL query strings (private — use DaemonClient methods instead).
# ---------------------------------------------------------------------------

_Q_ADD_MARKETPLACE = """
mutation AddMarketplace($input: MarketplaceSourceInput!) {
    addMarketplace(input: $input) {
        id name pluginCount error
        plugins {
            id name description
            skills { name fullName }
        }
    }
}
"""

_Q_REMOVE_MARKETPLACE = """
mutation RemoveMarketplace($id: ID!) {
    deleteNode(id: $id) {
        id
        ... on Marketplace { name }
    }
}
"""

_Q_LIST_MARKETPLACES = """
{ marketplaces { id name pluginCount error plugins { id name } } }
"""

_Q_INSTALL_PLUGIN = """
mutation InstallPlugin($input: PluginInstallInput!) {
    installPlugin(input: $input) {
        id name version
        installLocation { absolutePath }
        installedAt
    }
}
"""

_Q_UNINSTALL_PLUGIN = """
mutation UninstallPlugin($input: PluginInstallInput!) {
    uninstallPlugin(input: $input) { id name }
}
"""

_Q_SKILLS = """
{ skills { fullName plugin sourceKind } }
"""

_Q_GET_ENTRY = """
query GetEntry($path: String!) {
    getEntry(path: $path) {
        name type permissions
    }
}
"""

_Q_NEW_CHAT = """
mutation NewChat($model: ChatModel) {
    newChat(model: $model) { id status }
}
"""

_Q_SEND_MESSAGE = """
mutation SendMessage($input: SendMessageInput!) {
    sendMessage(input: $input) {
        ... on SendMessageProcessing { chatId executionId }
        ... on SendMessageError { chatId code message }
    }
}
"""

# The chat schema is now interface-driven: ToolMessagePart is an interface
# with one concrete type per tool kind, each carrying its own typed args.
# We pull the common fields from the interface plus type-specific args via
# inline fragments. The chat parser synthesizes a stable {name, input, output}
# shape from these so YAML regexes (input_matches against a JSON blob) keep
# working across the realign.
_PART_FIELDS = """
__typename partId
... on TextMessagePart { content }
... on ReasoningMessagePart { content }
... on ToolMessagePart {
    toolCallId toolName state
    output {
        __typename
        ... on TextToolOutput { content }
        ... on ErrorToolOutput { message }
        ... on ImageToolOutput { mimeType }
    }
}
... on ShellToolMessagePart { command note }
... on ReadFileToolMessagePart { path offset limit }
... on WriteFileToolMessagePart { path content }
... on EditFileToolMessagePart { filePath oldString newString replaceAll }
... on SkillToolMessagePart { skill resolvedSkillName }
... on SpawnAgentToolMessagePart { subagentType prompt note maxTurns }
... on ExtractToolMessagePart { path query schemaName }
"""

_Q_GET_CHAT = """
query GetChat($id: ID!) {
    node(id: $id) {
        ... on Chat {
            id status
            messages {
                __typename id status
                ... on AiChatMessage {
                    parts { %s }
                }
            }
        }
    }
}
""" % _PART_FIELDS

_Q_GET_TOOL_CHAIN = """
query GetToolChain($id: ID!) {
    node(id: $id) {
        ... on ToolChain {
            id toolCount
            messages {
                __typename
                ... on AiChatMessage {
                    parts { %s }
                }
            }
        }
    }
}
""" % _PART_FIELDS


class DaemonClient:
    def __init__(self, base_url: str, token: str) -> None:
        self.base_url = base_url.rstrip("/")
        self.graphql_url = f"{self.base_url}/graphql"
        self.health_url = f"{self.base_url}/actuator/health"
        self.auth_header = f"Bearer {token}"

    # -------------------------------------------------- transport
    def health_check(self) -> bool:
        try:
            req = urllib.request.Request(self.health_url)
            with urllib.request.urlopen(req, timeout=5) as resp:
                return json.loads(resp.read()).get("status") == "UP"
        except Exception:
            return False

    def gql(self, query: str, variables: dict | None = None) -> dict[str, Any]:
        payload = json.dumps({"query": query, "variables": variables or {}}).encode()
        req = urllib.request.Request(
            self.graphql_url,
            data=payload,
            headers={
                "Content-Type": "application/json",
                "Authorization": self.auth_header,
            },
        )
        with urllib.request.urlopen(req, timeout=120) as resp:
            return json.loads(resp.read())

    def gql_data(self, query: str, variables: dict | None = None) -> dict[str, Any]:
        result = self.gql(query, variables)
        if result.get("errors"):
            msg = result["errors"][0].get("message", str(result["errors"][0]))
            raise RuntimeError(msg)
        return result["data"]

    # -------------------------------------------------- filesystem introspection
    def get_entry(self, path: str) -> dict | None:
        """Returns {name, isFile, isDirectory, isSymlink} or None.

        The daemon's FileEntry now exposes a single `type: EntryType` enum
        instead of the legacy boolean flags; we translate back. The exec
        bit is no longer surfaced by the API, so callers should rely on
        plugin install hooks (which fail loudly on a chmod error) instead
        of asserting on file mode.
        """
        data = self.gql_data(_Q_GET_ENTRY, {"path": path})
        entry = data.get("getEntry")
        if not entry:
            return None
        kind = entry.get("type")
        return {
            "name": entry.get("name"),
            "isFile": kind == "FILE",
            "isDirectory": kind == "DIRECTORY",
            "isSymlink": kind == "SYMLINK",
        }

    # -------------------------------------------------- marketplace CRUD
    def add_marketplace(self, git_url: str, git_ref: str) -> dict:
        """Add a GIT-sourced marketplace. Returns the single Marketplace object.

        Raises RuntimeError if the daemon already has this marketplace —
        use `add_marketplace_idempotent` if you want auto-retry.
        """
        data = self.gql_data(
            _Q_ADD_MARKETPLACE,
            {"input": {"sourceType": "GIT", "gitUrl": git_url, "gitRef": git_ref}},
        )
        return data["addMarketplace"]

    def add_marketplace_idempotent(self, git_url: str, git_ref: str,
                                   manifest: dict) -> dict:
        """Like add_marketplace, but removes any stale copy (matched by name
        from the local manifest) and retries once on 'already configured'.

        Uses the daemon-issued node id from `list_marketplaces` rather than
        computing it locally — the daemon's NodeId wire format (protobuf
        varint + URL-safe base64) isn't worth re-implementing here.
        """
        try:
            return self.add_marketplace(git_url, git_ref)
        except RuntimeError as e:
            if "already configured" not in str(e).lower():
                raise
            try:
                stale = next(
                    (m for m in self.list_marketplaces()
                     if m["name"] == manifest["name"]),
                    None,
                )
                if stale:
                    self.remove_marketplace(stale["id"])
            except Exception:
                pass
            return self.add_marketplace(git_url, git_ref)

    def remove_marketplace(self, node_id: str) -> dict | None:
        data = self.gql_data(_Q_REMOVE_MARKETPLACE, {"id": node_id})
        return data.get("deleteNode")

    def list_marketplaces(self) -> list[dict]:
        data = self.gql_data(_Q_LIST_MARKETPLACES)
        return data["marketplaces"]

    # -------------------------------------------------- plugins
    def install_plugin(self, plugin_id: str) -> dict:
        data = self.gql_data(_Q_INSTALL_PLUGIN, {"input": {"id": plugin_id}})
        return data["installPlugin"]

    def uninstall_plugin(self, plugin_id: str) -> dict:
        data = self.gql_data(_Q_UNINSTALL_PLUGIN, {"input": {"id": plugin_id}})
        return data["uninstallPlugin"]

    # -------------------------------------------------- skills
    def list_skills(self) -> list[dict]:
        data = self.gql_data(_Q_SKILLS)
        return data["skills"]

    # -------------------------------------------------- chat
    def new_chat(self, model: str | None = None) -> dict:
        data = self.gql_data(_Q_NEW_CHAT, {"model": model})
        return data["newChat"]

    def send_message(self, chat_id: str, message: str) -> dict:
        data = self.gql_data(
            _Q_SEND_MESSAGE,
            {"input": {"chatId": chat_id, "message": message}},
        )
        return data["sendMessage"]

    def get_chat(self, node_id: str) -> dict:
        data = self.gql_data(_Q_GET_CHAT, {"id": node_id})
        return data["node"] or {}

    def get_tool_chain(self, node_id: str) -> dict:
        data = self.gql_data(_Q_GET_TOOL_CHAIN, {"id": node_id})
        return data["node"] or {}

    # -------------------------------------------------- file upload
    def upload_folder(self, destination: str,
                      files: list[tuple[str, bytes | str]]) -> dict:
        """POST /rest/files/upload-folder — places files preserving folder structure.

        `destination` is a path inside the VM (e.g. '/home/codexis/ee-mkt-xyz').
        `files` is a list of (relative_path, content) — content can be bytes
        or str. Uses a hand-rolled multipart/form-data body so the test suite
        doesn't pick up `requests` as a dependency.
        """
        boundary = "----ee2e" + uuid.uuid4().hex
        body = bytearray()

        def _field(name: str, value: str) -> None:
            body.extend(f"--{boundary}\r\n".encode())
            body.extend(
                f'Content-Disposition: form-data; name="{name}"\r\n\r\n'.encode()
            )
            body.extend(value.encode())
            body.extend(b"\r\n")

        def _file(name: str, filename: str, content: bytes) -> None:
            body.extend(f"--{boundary}\r\n".encode())
            body.extend(
                f'Content-Disposition: form-data; name="{name}"; '
                f'filename="{filename}"\r\n'.encode()
            )
            body.extend(b"Content-Type: application/octet-stream\r\n\r\n")
            body.extend(content)
            body.extend(b"\r\n")

        _field("destination", destination)
        for rel, content in files:
            if isinstance(content, str):
                content = content.encode()
            _file("files", rel, content)
            _field("relativePaths", rel)
        body.extend(f"--{boundary}--\r\n".encode())

        req = urllib.request.Request(
            f"{self.base_url}/rest/files/upload-folder",
            data=bytes(body),
            headers={
                "Authorization": self.auth_header,
                "Content-Type": f"multipart/form-data; boundary={boundary}",
            },
        )
        with urllib.request.urlopen(req, timeout=60) as resp:
            return json.loads(resp.read())

    # -------------------------------------------------- file download
    def download_file(self, path: str) -> bytes:
        """Fetch a VM-side file's raw bytes via REST.

        Used by test-marketplace.py to read the daemon-managed .env file
        and verify each plugin's required vars are populated.
        """
        url = f"{self.base_url}/rest/files/download?path={urllib.parse.quote(path)}"
        req = urllib.request.Request(url, headers={"Authorization": self.auth_header})
        with urllib.request.urlopen(req, timeout=30) as resp:
            return resp.read()

    def run_single_shot_chat(self, prompt: str, model: str | None = None,
                             poll_interval_s: float = 2.0,
                             poll_timeout_s: float = 120.0) -> str:
        """Start a fresh chat, send one prompt, return the assistant's final text."""
        info = self.new_chat(model)
        self.send_message(info["id"], prompt)
        deadline = time.monotonic() + poll_timeout_s
        while True:
            chat = self.get_chat(info["id"])
            status = chat.get("status")
            if status == "ERROR":
                raise RuntimeError(f"chat ERROR: {chat}")
            if status == "READY" and chat.get("messages"):
                msg = chat["messages"][-1]
                parts = msg.get("parts") or []
                return "".join(
                    p.get("content") or "" for p in parts
                    if p.get("__typename") == "TextMessagePart"
                )
            if time.monotonic() >= deadline:
                raise RuntimeError(f"chat timed out after {poll_timeout_s}s")
            time.sleep(poll_interval_s)
