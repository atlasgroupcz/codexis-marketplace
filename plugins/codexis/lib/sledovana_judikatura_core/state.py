"""State I/O: topic.json + per-report JSON files, atomic writes.

Directory layout under APP_DIR:

    <uuid>/
        topic.json
        reports/
            <report_id>.json
"""

import datetime
import json
import os
import tempfile

_USER_HOME = os.environ.get("CDX_USER_HOME") or os.path.expanduser("~")
APP_DIR = os.path.join(_USER_HOME, ".cdx", "apps", "sledovana-judikatura")


def now_utc():
    """Current UTC timestamp in ISO 8601 'Z' form."""
    return datetime.datetime.now(datetime.timezone.utc).isoformat(
        timespec="seconds"
    ).replace("+00:00", "Z")


# ── atomic writes ────────────────────────────────────────────────────────────


def atomic_write_json(path, data):
    """Write JSON atomically (tmp file + rename)."""
    d = os.path.dirname(path) or "."
    os.makedirs(d, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=".tmp.", suffix=".json", dir=d)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
            f.write("\n")
        os.replace(tmp, path)
    except Exception:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


# ── path helpers ─────────────────────────────────────────────────────────────


def topic_dir(topic_uuid):
    return os.path.join(APP_DIR, topic_uuid)


def topic_path(topic_uuid):
    return os.path.join(topic_dir(topic_uuid), "topic.json")


def reports_dir(topic_uuid):
    return os.path.join(topic_dir(topic_uuid), "reports")


def report_path(topic_uuid, report_id):
    return os.path.join(reports_dir(topic_uuid), f"{report_id}.json")


# ── topic I/O ────────────────────────────────────────────────────────────────


def load_topic(topic_uuid):
    """Return topic dict, or None if missing or corrupt."""
    path = topic_path(topic_uuid)
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError):
        return None


def save_topic(topic_uuid, data):
    atomic_write_json(topic_path(topic_uuid), data)


def all_topics():
    """Return list of (uuid, topic_data) for all topics with readable topic.json."""
    if not os.path.isdir(APP_DIR):
        return []
    results = []
    for name in sorted(os.listdir(APP_DIR)):
        data = load_topic(name)
        if data is not None:
            results.append((name, data))
    return results


# ── uuid resolution ──────────────────────────────────────────────────────────


def resolve_uuid(partial):
    """Resolve a partial UUID to a full one.

    Returns:
        str: full UUID on unique match (including exact match)
        None: no match
    Raises:
        AmbiguousTopicPrefixError: prefix matches multiple topics
    """
    from .exceptions import AmbiguousTopicPrefixError

    if os.path.isdir(topic_dir(partial)):
        return partial
    if not os.path.isdir(APP_DIR):
        return None
    matches = [d for d in os.listdir(APP_DIR) if d.startswith(partial)]
    if len(matches) == 1:
        return matches[0]
    if len(matches) > 1:
        raise AmbiguousTopicPrefixError(
            f"Nejednoznačný UUID prefix '{partial}', "
            f"nalezeno {len(matches)} shod"
        )
    return None


# ── report I/O ───────────────────────────────────────────────────────────────


def load_report(topic_uuid, report_id):
    """Return report dict, or None if missing or corrupt."""
    path = report_path(topic_uuid, report_id)
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError):
        return None


def save_report(topic_uuid, report_id, data):
    atomic_write_json(report_path(topic_uuid, report_id), data)


def all_reports(topic_uuid):
    """Return list of full report dicts for a topic, sorted by report_id."""
    rd = reports_dir(topic_uuid)
    if not os.path.isdir(rd):
        return []
    reports = []
    for rf in sorted(os.listdir(rd)):
        if not rf.endswith(".json"):
            continue
        try:
            with open(os.path.join(rd, rf), "r", encoding="utf-8") as f:
                reports.append(json.load(f))
        except (json.JSONDecodeError, OSError):
            continue
    return reports


def report_summaries(topic_uuid):
    """Return list of lightweight report summary dicts, sorted by report_id."""
    rd = reports_dir(topic_uuid)
    if not os.path.isdir(rd):
        return []
    summaries = []
    for rf in sorted(os.listdir(rd)):
        if not rf.endswith(".json"):
            continue
        try:
            with open(os.path.join(rd, rf), "r", encoding="utf-8") as f:
                r = json.load(f)
        except (json.JSONDecodeError, OSError):
            continue
        summaries.append({
            "report_id": rf.replace(".json", ""),
            "checked_at": r.get("checked_at"),
            "period_from": r.get("period_from"),
            "period_to": r.get("period_to"),
            "found_count": r.get("found_count", 0),
            "confirmed_on": r.get("confirmed_on"),
        })
    return summaries
