"""GraphQL client for cdx-daemon. Shared by acceptance and e2e test suites."""

from __future__ import annotations

import base64
import json
import urllib.request
from typing import Any


def encode_node_id(type_name: str, identifier: str) -> str:
    return base64.b64encode(f"{type_name}:{identifier}".encode()).decode()


class DaemonClient:
    def __init__(self, base_url: str, token: str) -> None:
        self.base_url = base_url.rstrip("/")
        self.graphql_url = f"{self.base_url}/graphql"
        self.health_url = f"{self.base_url}/actuator/health"
        self.auth_header = f"Bearer {token}"

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

    def get_entry(self, path: str) -> dict | None:
        data = self.gql_data(GET_ENTRY, {"path": path})
        return data.get("getEntry")

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


ADD_MARKETPLACE = """
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

REMOVE_MARKETPLACE = """
mutation RemoveMarketplace($id: ID!) {
    removeMarketplace(id: $id) { id name }
}
"""

LIST_MARKETPLACES = """
{ marketplaces { id name pluginCount error plugins { id name } } }
"""

INSTALL_PLUGIN = """
mutation InstallPlugin($input: PluginInstallInput!) {
    installPlugin(input: $input) {
        id name version
        installLocation { absolutePath }
        installedAt
    }
}
"""

UNINSTALL_PLUGIN = """
mutation UninstallPlugin($input: PluginInstallInput!) {
    uninstallPlugin(input: $input) { id name }
}
"""

SKILLS = """
{ skills { fullName plugin sourceKind } }
"""

GET_ENTRY = """
query GetEntry($path: String!) {
    getEntry(path: $path) {
        name isFile isDirectory isSymlink executable
    }
}
"""
