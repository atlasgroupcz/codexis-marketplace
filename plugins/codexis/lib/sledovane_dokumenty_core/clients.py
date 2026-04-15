"""External process clients: cdx-cli (CODEXIS access) and cdx-daemon LLM extract."""

import json
import os
import subprocess
import tempfile

from .exceptions import CdxClientError, LlmDaemonError

CDX_CLI_BIN = "cdx-cli"
DEFAULT_DAEMON_URL = "http://localhost:8086"
DAEMON_ENV_FILE = os.path.expanduser("~/.cdx/.daemon.env")


# ── cdx-cli wrapper ──────────────────────────────────────────────────────────


def cdx_get(path):
    """Call `cdx-cli get <path>` and return parsed JSON (or raw text if non-JSON).

    Raises CdxClientError on subprocess failure or API-level error.
    """
    result = subprocess.run(
        [CDX_CLI_BIN, "get", path],
        capture_output=True, text=True, timeout=30,
    )
    if result.returncode != 0:
        raise CdxClientError(
            f"cdx-cli get failed for {path}: {result.stderr.strip()}"
        )
    stdout = result.stdout.strip()
    try:
        data = json.loads(stdout)
    except json.JSONDecodeError:
        return stdout
    if isinstance(data, dict) and "error" in data:
        raise CdxClientError(f"CODEXIS API error for {path}: {data['error']}")
    return data


def cdx_get_text(path):
    """Call `cdx-cli get <path>` and return raw text body.

    Raises CdxClientError on subprocess failure or API-level error.
    """
    result = subprocess.run(
        [CDX_CLI_BIN, "get", path],
        capture_output=True, text=True, timeout=30,
    )
    if result.returncode != 0:
        raise CdxClientError(
            f"cdx-cli get failed for {path}: {result.stderr.strip()}"
        )
    stdout = result.stdout.strip()
    if stdout.startswith("{"):
        try:
            data = json.loads(stdout)
            if isinstance(data, dict) and "error" in data:
                raise CdxClientError(
                    f"CODEXIS API error for {path}: {data['error']}"
                )
        except json.JSONDecodeError:
            pass
    return result.stdout


# ── high-level fetch helpers ─────────────────────────────────────────────────


def get_versions(codexis_id):
    return cdx_get(f"cdx://doc/{codexis_id}/versions")


def get_meta(codexis_id):
    return cdx_get(f"cdx://doc/{codexis_id}/meta")


def get_latest_version_id(versions):
    if not versions or not isinstance(versions, list):
        return None
    return versions[0].get("versionId")


def get_doc_name(meta):
    if not isinstance(meta, dict):
        return None
    cr = meta.get("cr", {})
    main = cr.get("main", {})
    return main.get("title")


def find_version_info(versions, version_id):
    for v in versions:
        if v.get("versionId") == version_id:
            return v
    return None


def resolve_amendments(versions, latest_vid):
    """Resolve amendment IDs to {id, name} dicts for a given version."""
    version_info = find_version_info(versions, latest_vid)
    amendment_ids = version_info.get("amendmentDocIds", []) if version_info else []
    amendments = []
    for aid in amendment_ids:
        base_id = aid.split("_")[0] if "_" in aid else aid
        try:
            meta = get_meta(base_id)
        except CdxClientError:
            meta = None
        title = get_doc_name(meta) if meta else None
        amendments.append({"id": aid, "name": title or aid})
    return amendments


def fetch_doc_title(doc_id):
    """Fetch title for a document, falling back to doc_id on failure."""
    base_id = doc_id.split("_")[0] if "_" in doc_id else doc_id
    try:
        meta = get_meta(base_id)
    except CdxClientError:
        return doc_id
    if meta:
        title = get_doc_name(meta)
        if title:
            return title
    return doc_id


def fetch_all_related_ids(codexis_id, relation_type):
    """Fetch ALL related doc IDs for a type (paginated). Returns sorted list.

    Raises CdxClientError on subprocess / API failure.
    """
    all_ids = []
    offset = 0
    limit = 100
    while True:
        data = cdx_get(
            f"cdx://doc/{codexis_id}/related"
            f"?type={relation_type}&limit={limit}&offset={offset}"
        )
        if not isinstance(data, dict):
            raise CdxClientError(
                f"Unexpected response for related {relation_type}: {data!r}"
            )
        results = data.get("results", [])
        if not results:
            break
        for item in results:
            doc_id = item.get("docId") if isinstance(item, dict) else item
            if doc_id:
                all_ids.append(str(doc_id))
        total = data.get("totalResults", 0)
        offset += limit
        if offset >= total:
            break
    return sorted(set(all_ids))


def fetch_related_counts(codexis_id):
    """Fetch related counts by type. Returns list of {type, typeName, count}."""
    data = cdx_get(f"cdx://doc/{codexis_id}/related/counts")
    if isinstance(data, dict):
        counts = data.get("counts", [])
        result = []
        for item in counts:
            result.append({
                "type": item.get("type"),
                "typeName": item.get("name", item.get("typeName", item.get("type"))),
                "count": item.get("count", 0),
            })
        return result
    if isinstance(data, list):
        return data
    raise CdxClientError(f"Unexpected related counts response: {data!r}")


# ── LLM daemon client ────────────────────────────────────────────────────────


def load_daemon_auth():
    """Load CDX_DAEMON_AUTH from env or ~/.cdx/.daemon.env."""
    val = os.environ.get("CDX_DAEMON_AUTH", "")
    if val:
        return val
    try:
        with open(DAEMON_ENV_FILE) as f:
            for line in f:
                if line.startswith("CDX_DAEMON_AUTH="):
                    val = line[len("CDX_DAEMON_AUTH="):].strip()
                    if val:
                        return val
    except OSError:
        pass
    return ""


def llm_extract(text, query):
    """Call /rest/llm/extract on the daemon. Returns response text, or None on failure.

    Returns None rather than raising so callers can mark summaries as pending
    and retry next check cycle.
    """
    daemon_url = os.environ.get("CDX_DAEMON_URL", DEFAULT_DAEMON_URL)
    daemon_auth = load_daemon_auth()
    if not daemon_auth:
        return None

    tmp_path = None
    try:
        # Write text to temp file to avoid curl -F truncation with special chars.
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write(text)
            tmp_path = f.name
        cmd = [
            "curl", "-sS", "-X", "POST",
            f"{daemon_url}/rest/llm/extract",
            "-H", f"Authorization: {daemon_auth}",
            "-F", f"text=<{tmp_path}",
            "-F", f"query={query}",
        ]
        chat_id = os.environ.get("CDX_SESSION_ID")
        if chat_id:
            cmd.extend(["-H", f"X-CDX-Session-Id: {chat_id}"])
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=120,
        )
        if result.returncode != 0:
            return None
        try:
            data = json.loads(result.stdout)
        except json.JSONDecodeError:
            return None
        return data.get("response")
    except (subprocess.SubprocessError, OSError):
        return None
    finally:
        if tmp_path:
            try:
                os.unlink(tmp_path)
            except OSError:
                pass
