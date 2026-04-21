"""GraphQL client for cdx-daemon. Shared by acceptance and e2e test suites."""

from __future__ import annotations

import base64
import json
import time
import urllib.request
from typing import Any


def encode_node_id(type_name: str, identifier: str) -> str:
    """Build a base64 Relay-style node ID (e.g. for Marketplace lookups)."""
    return base64.b64encode(f"{type_name}:{identifier}".encode()).decode()


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
    removeMarketplace(id: $id) { id name }
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
        name isFile isDirectory isSymlink executable
    }
}
"""

_Q_NEW_CHAT = """
mutation NewChat($model: ChatModel) {
    newChat(model: $model) { id chatId status }
}
"""

_Q_SEND_MESSAGE = """
mutation SendMessage($chatId: ID!, $message: String!) {
    sendMessage(chatId: $chatId, message: $message) {
        chatId
        ... on SendMessageProcessing { executionId }
        ... on SendMessageError { message }
    }
}
"""

_Q_GET_CHAT = """
query GetChat($id: ID!) {
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
}
"""


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
        data = self.gql_data(_Q_GET_ENTRY, {"path": path})
        return data.get("getEntry")

    # -------------------------------------------------- marketplace CRUD
    def add_marketplace(self, git_url: str, git_ref: str) -> list[dict]:
        """Add a GIT-sourced marketplace. Returns the full `addMarketplace` list.

        Raises RuntimeError if the daemon already has this marketplace —
        use `add_marketplace_idempotent` if you want auto-retry.
        """
        data = self.gql_data(
            _Q_ADD_MARKETPLACE,
            {"input": {"sourceType": "GIT", "gitUrl": git_url, "gitRef": git_ref}},
        )
        return data["addMarketplace"]

    def add_marketplace_idempotent(self, git_url: str, git_ref: str,
                                   manifest: dict) -> list[dict]:
        """Like add_marketplace, but removes any stale copy (matched by uuid
        from the local manifest) and retries once on 'already configured'."""
        try:
            return self.add_marketplace(git_url, git_ref)
        except RuntimeError as e:
            if "already configured" not in str(e).lower():
                raise
            try:
                self.remove_marketplace(
                    encode_node_id("Marketplace", manifest.get("uuid", ""))
                )
            except Exception:
                pass
            return self.add_marketplace(git_url, git_ref)

    def remove_marketplace(self, node_id: str) -> dict | None:
        data = self.gql_data(_Q_REMOVE_MARKETPLACE, {"id": node_id})
        return data.get("removeMarketplace")

    def list_marketplaces(self) -> list[dict]:
        data = self.gql_data(_Q_LIST_MARKETPLACES)
        return data["marketplaces"]

    # -------------------------------------------------- plugins
    def install_plugin(self, plugin_id: str) -> list[dict]:
        data = self.gql_data(_Q_INSTALL_PLUGIN, {"input": {"id": plugin_id}})
        return data["installPlugin"]

    def uninstall_plugin(self, plugin_id: str) -> list[dict]:
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
        data = self.gql_data(_Q_SEND_MESSAGE, {"chatId": chat_id, "message": message})
        return data["sendMessage"]

    def get_chat(self, node_id: str) -> dict:
        data = self.gql_data(_Q_GET_CHAT, {"id": node_id})
        return data["node"] or {}

    def run_single_shot_chat(self, prompt: str, model: str | None = None,
                             poll_interval_s: float = 2.0,
                             poll_timeout_s: float = 120.0) -> str:
        """Start a fresh chat, send one prompt, return the assistant's final text."""
        info = self.new_chat(model)
        self.send_message(info["chatId"], prompt)
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
