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
