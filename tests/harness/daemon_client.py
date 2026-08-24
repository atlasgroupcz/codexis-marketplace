"""GraphQL client for cdx-daemon. Shared by acceptance and e2e test suites."""

from __future__ import annotations

import json
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid
from collections.abc import Callable
from typing import Any


def make_oauth2_proxy_refresher(
    cookie: str, oauth2_proxy_url: str = "http://localhost:4182"
):
    """Return a zero-arg callable that mints a fresh bearer via oauth2-proxy.

    Sends the user's `_oauth2_proxy` session cookie to /oauth2/auth and reads
    the access token from the X-Auth-Request-Access-Token response header.
    Used by long-running suites to ride past the keycloak token TTL (~30 min).
    """
    auth_url = f"{oauth2_proxy_url.rstrip('/')}/oauth2/auth"

    def _refresh() -> str | None:
        req = urllib.request.Request(auth_url, headers={"Cookie": cookie})
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                return resp.headers.get("X-Auth-Request-Access-Token")
        except Exception:
            return None

    return _refresh


def _normalize_git_url(git_url: str) -> str:
    """Canonical form for comparing two git URLs: lowercased, no trailing
    slash, no `.git` suffix. Mirrors the daemon's own normalization so a record
    fetched from `marketplaces` matches the URL we are about to add."""
    u = (git_url or "").strip().lower().rstrip("/")
    return u[:-4] if u.endswith(".git") else u


def _git_url_basename(git_url: str) -> str:
    """Last path segment of a git URL, `.git` stripped, lowercased - the name
    the daemon falls back to (`deriveSourceKey`) when it can't read a manifest."""
    return _normalize_git_url(git_url).rsplit("/", 1)[-1]


# ---------------------------------------------------------------------------
# GraphQL query strings (private - use DaemonClient methods instead).
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
{ marketplaces { id name deletable pluginCount error source { url ref } plugins { id name } } }
"""

_Q_INSTALL_PLUGIN = """
mutation InstallPlugin($input: PluginInstallInput!) {
    installPlugin(input: $input) {
        id name version
        installation { installedAt installedVersion }
    }
}
"""

_Q_UNINSTALL_PLUGIN = """
mutation UninstallPlugin($input: PluginInstallInput!) {
    uninstallPlugin(input: $input) { id name }
}
"""

_Q_SKILLS = """
{ skills { fullName plugin { id name } sourceKind } }
"""

# The VM lifecycle. Both go to the gateway endpoint, not the daemon one.
_Q_START_VM = """
mutation StartVirtualMachine {
    startVirtualMachine {
        __typename
        ... on VirtualMachine { status }
        ... on ActionError { message }
    }
}
"""

_Q_VM_STATUS = """
{ me { virtualMachine { status } } }
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
... on ReasoningMessagePart { reasoningContent: content }
... on ToolMessagePart {
    toolCallId toolName state
    output {
        blocks {
            __typename
            ... on TextToolOutput { content }
            ... on ErrorToolOutput { message }
            ... on ImageToolOutput { mimeType }
        }
    }
    sources {
        __typename
        ... on UrlSource { url title snippet }
        ... on FileSource { path { displayPath } }
    }
}
... on ShellToolMessagePart { command note }
... on ReadFileToolMessagePart { path offset limit }
... on WriteFileToolMessagePart { path fileContent: content }
... on EditFileToolMessagePart { filePath oldString newString replaceAll }
... on SkillToolMessagePart { skill }
... on SpawnAgentToolMessagePart { subagentType prompt note maxTurns }
... on ExtractToolMessagePart { path query schemaName }
"""

_Q_GET_CHAT = (
    """
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
"""
    % _PART_FIELDS
)

_Q_GET_TOOL_CHAIN = (
    """
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
"""
    % _PART_FIELDS
)


# HTTP statuses worth retrying. The shared dev daemon redeploys under long runs,
# so a single request can momentarily hit a gateway 405/5xx while the backend
# reloads - transient, not a test failure.
_TRANSIENT_STATUS = frozenset({405, 408, 429, 500, 502, 503, 504})
_MAX_ATTEMPTS = 5


def _discard(_message: str) -> None:
    """Default sink for ensure_vm_running's progress log."""


class DaemonClient:
    def __init__(
        self,
        base_url: str,
        token: str,
        token_refresher: Callable[[], str | None] | None = None,
    ) -> None:
        """`token_refresher` is an optional zero-arg callable that returns a
        fresh bearer token. When set, the client transparently refreshes on
        HTTP 401 once per failed request - keeps long suites alive past the
        keycloak access-token TTL (~30 min).
        """
        self.base_url = base_url.rstrip("/")
        self.graphql_url = f"{self.base_url}/graphql"
        # The VM lifecycle lives on the gateway, not the daemon: the daemon runs
        # inside the VM this endpoint starts, so it cannot be the one asked to
        # start it.
        self.gateway_graphql_url = f"{self.base_url}/gateway/graphql"
        self.health_url = f"{self.base_url}/actuator/health"
        self.auth_header = f"Bearer {token}"
        self._token_refresher = token_refresher

    # -------------------------------------------------- transport
    def health_check(self) -> bool:
        try:
            req = urllib.request.Request(
                self.health_url, headers={"Authorization": self.auth_header}
            )
            with urllib.request.urlopen(req, timeout=5) as resp:
                return json.loads(resp.read()).get("status") == "UP"
        except Exception:
            return False

    def wait_healthy(
        self,
        timeout_s: float = 90,
        interval_s: float = 3,
        sleep: Callable[[float], None] = time.sleep,
        now: Callable[[], float] = time.monotonic,
    ) -> bool:
        """Poll the health endpoint until the daemon reports UP or the deadline
        passes. The shared dev daemon redeploys under long runs and can be
        briefly down exactly when the suite starts, so wait it out instead of
        aborting preflight on the first probe. sleep/now are injectable for tests.
        """
        deadline = now() + timeout_s
        while True:
            if self.health_check():
                return True
            if now() >= deadline:
                return False
            sleep(interval_s)

    # -------------------------------------------------- virtual machine
    def start_vm(self) -> dict:
        """Ask the gateway to start this user's VM.

        Returns the union member it answered with - `{"__typename":
        "VirtualMachine", "status": ...}` on success, `{"__typename":
        "ActionError", "message": ...}` when it declined.
        """
        data = self.gql_data(_Q_START_VM, url=self.gateway_graphql_url)
        return data.get("startVirtualMachine") or {}

    def vm_status(self) -> str:
        """This user's VM status, or "UNKNOWN" when the gateway reports none."""
        data = self.gql_data(_Q_VM_STATUS, url=self.gateway_graphql_url)
        vm = (data.get("me") or {}).get("virtualMachine") or {}
        return vm.get("status") or "UNKNOWN"

    def ensure_vm_running(
        self,
        start_attempts: int = 5,
        timeout_s: float = 600,
        interval_s: float = 5,
        settle_s: float = 60,
        sleep: Callable[[float], None] = time.sleep,
        now: Callable[[], float] = time.monotonic,
        log: Callable[[str], None] | None = None,
    ) -> None:
        """Start this user's VM if needed and block until it reports RUNNING.

        Installing a plugin writes into the VM, so a suite that runs against a
        stopped account has nowhere to install to. Raises RuntimeError if the VM
        cannot be started, reaches a terminal STOPPED, or does not come up in
        time. sleep/now are injectable for tests.

        The start is retried with a short exponential backoff: the VM service
        answers a "temporarily unavailable" ActionError or a 5xx while it
        redeploys, which is a blip rather than a refusal. Poll failures are
        tolerated for the same reason - only a terminal state or the deadline
        ends the wait.

        STOPPED is both the state a VM starts from and the state a failed start
        lands in, so it only counts as terminal once the VM has been seen active
        or settle_s has passed since the start was accepted. The gateway reports
        STOPPED for the moment between accepting the start and the control plane
        publishing the new instance, and aborting on that first poll fails a run
        whose VM goes on to boot normally.
        """
        say = log if log is not None else _discard

        for attempt in range(1, start_attempts + 1):
            try:
                started = self.start_vm()
            except RuntimeError as exc:
                started = {"__typename": "ActionError", "message": str(exc)}
            if started.get("__typename") == "VirtualMachine":
                break
            if attempt == start_attempts:
                raise RuntimeError(
                    f"startVirtualMachine did not succeed after {start_attempts} "
                    f"attempts: {started.get('message') or started!r}"
                )
            backoff = 2 ** (attempt - 1)
            say(
                f"  VM start not accepted (attempt {attempt}/{start_attempts}), "
                f"retrying in {backoff}s: {started.get('message') or started!r}"
            )
            sleep(backoff)

        say(f"  VM start accepted (status {started.get('status') or 'unknown'})")

        deadline = now() + timeout_s
        settle_deadline = now() + settle_s
        seen_active = False
        while True:
            try:
                state = self.vm_status()
            except RuntimeError:
                state = "REQUEST_FAILED"
            say(f"  VM state: {state}")
            if state == "RUNNING":
                return
            if state == "STOPPED":
                if seen_active or now() >= settle_deadline:
                    raise RuntimeError(
                        "VM entered terminal state STOPPED while waiting for RUNNING"
                    )
            elif state != "REQUEST_FAILED":
                seen_active = True
            if now() >= deadline:
                raise RuntimeError(
                    f"VM did not reach RUNNING within {timeout_s}s (last state: {state})"
                )
            sleep(interval_s)

    def _refresh_token(self) -> bool:
        if self._token_refresher is None:
            return False
        try:
            new_token = self._token_refresher()
        except Exception:
            return False
        if not new_token:
            return False
        self.auth_header = f"Bearer {new_token}"
        return True

    def _open(self, req: urllib.request.Request, timeout: float = 120):
        """urlopen with a 401-driven token refresh and bounded retries.

        The shared dev daemon redeploys under long runs, so a single request can
        momentarily hit a gateway 405/5xx (see _TRANSIENT_STATUS) or a dropped
        connection while the backend reloads. Those are infrastructure blips, not
        test failures, so retry with a short backoff before giving up. A 401 is
        handled separately: refresh the token once and retry immediately.
        """
        last_exc: urllib.error.URLError | None = None
        refreshed = False
        for attempt in range(_MAX_ATTEMPTS):
            try:
                return urllib.request.urlopen(req, timeout=timeout)
            except urllib.error.HTTPError as exc:
                if exc.code == 401 and not refreshed and self._refresh_token():
                    refreshed = True
                    req.headers["Authorization"] = self.auth_header
                    continue  # immediate retry with the fresh token
                if exc.code not in _TRANSIENT_STATUS:
                    raise
                last_exc = exc
            except urllib.error.URLError as exc:
                last_exc = exc  # connection reset/refused while the daemon reloads
            if attempt < _MAX_ATTEMPTS - 1:
                time.sleep(min(2**attempt, 8))
        raise last_exc if last_exc is not None else RuntimeError("no request attempted")

    def gql(
        self, query: str, variables: dict | None = None, url: str | None = None
    ) -> dict[str, Any]:
        payload = json.dumps({"query": query, "variables": variables or {}}).encode()
        req = urllib.request.Request(
            url or self.graphql_url,
            data=payload,
            headers={
                "Content-Type": "application/json",
                "Authorization": self.auth_header,
            },
        )
        with self._open(req, timeout=120) as resp:
            return json.loads(resp.read())

    def gql_data(
        self, query: str, variables: dict | None = None, url: str | None = None
    ) -> dict[str, Any]:
        result = self.gql(query, variables, url=url)
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

        Raises RuntimeError if the daemon already has this marketplace -
        use `add_marketplace_idempotent` if you want auto-retry.
        """
        data = self.gql_data(
            _Q_ADD_MARKETPLACE,
            {"input": {"sourceType": "GIT", "gitUrl": git_url, "gitRef": git_ref}},
        )
        return data["addMarketplace"]

    def add_marketplace_idempotent(
        self, git_url: str, git_ref: str, manifest: dict
    ) -> dict:
        """Add the marketplace from a clean slate: remove any existing copy of
        the same git URL first, then add it fresh. This guarantees a fresh
        checkout every run and side-steps a stale daemon record whose on-disk
        checkout went missing (which the daemon reports with pluginCount=0 and
        an error, never re-cloning on its own).

        Matching is on the source git URL, not the manifest name, on purpose:
        when the checkout is missing the daemon reports the marketplace under a
        URL-derived fallback name (e.g. `codexis-marketplace`) rather than the
        manifest name (`codexis-plugins`), so a name match would miss exactly
        the records we need to clear. `manifest` is kept only for the defensive
        name-based sweep below.

        Uses the daemon-issued node id from `list_marketplaces` rather than
        computing it locally - the daemon's NodeId wire format (protobuf
        varint + URL-safe base64) isn't worth re-implementing here.
        """
        target = _normalize_git_url(git_url)
        for m in self.list_marketplaces():
            # Skip records the daemon marks as protected (e.g. the builtin
            # marketplace). It reports `deletable=false` and rejects the delete
            # with a 400, which would otherwise abort the whole sweep. Ask the
            # daemon rather than guessing from the URL - the builtin shares this
            # repo's git URL when its checkout is stale, so URL matching alone
            # cannot tell them apart.
            if not m.get("deletable"):
                continue
            if _normalize_git_url((m.get("source") or {}).get("url") or "") == target:
                self.remove_marketplace(m["id"])

        try:
            return self.add_marketplace(git_url, git_ref)
        except RuntimeError as e:
            # Defensive: a record with no queryable source URL won't be caught
            # above, so fall back to a name sweep (manifest name + URL basename)
            # and retry once.
            if "already configured" not in str(e).lower():
                raise
            names = {manifest.get("name"), _git_url_basename(git_url)}
            for m in self.list_marketplaces():
                # Same protected-record guard as the URL sweep above: the
                # builtin's URL-basename fallback name can land in `names`.
                if not m.get("deletable"):
                    continue
                if m.get("name") in names:
                    self.remove_marketplace(m["id"])
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
    def upload_folder(
        self, destination: str, files: list[tuple[str, bytes | str]]
    ) -> dict:
        """POST /rest/files/upload-folder - places files preserving folder structure.

        `destination` is a path inside the VM (e.g. '/home/codexis/ee-mkt-xyz').
        `files` is a list of (relative_path, content) - content can be bytes
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
        with self._open(req, timeout=60) as resp:
            return json.loads(resp.read())

    # -------------------------------------------------- file download
    def download_file(self, path: str) -> bytes:
        """Fetch a VM-side file's raw bytes via REST.

        Used by the marketplace tier to read the daemon-managed .env file
        and verify each plugin's required vars are populated.
        """
        url = f"{self.base_url}/rest/files/download?path={urllib.parse.quote(path)}"
        req = urllib.request.Request(url, headers={"Authorization": self.auth_header})
        with self._open(req, timeout=30) as resp:
            return resp.read()

    def run_single_shot_chat(
        self,
        prompt: str,
        model: str | None = None,
        poll_interval_s: float = 2.0,
        poll_timeout_s: float = 120.0,
    ) -> str:
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
                    p.get("content") or ""
                    for p in parts
                    if p.get("__typename") == "TextMessagePart"
                )
            if time.monotonic() >= deadline:
                raise RuntimeError(f"chat timed out after {poll_timeout_s}s")
            time.sleep(poll_interval_s)
