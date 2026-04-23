"""High-level operations for case-law topic tracking.

Covers topic lifecycle, notes, areas, reports. Everything else (state I/O,
paths, uuid resolution) lives in state.py so this module focuses on business
rules and sequencing.
"""

import json
import os
import shutil
import subprocess
import sys
import uuid as uuid_mod

from . import state
from .exceptions import (
    AreaAlreadyExistsError,
    AreaIndexError,
    NoteIndexError,
    ReportNotFoundError,
    ReportSourceError,
    TopicNotFoundError,
)
from .state import now_utc

CDXCTL_BIN = "cdxctl"


def _require_topic(partial_uuid):
    """Resolve prefix and load topic. Raises TopicNotFoundError if missing."""
    full_uuid = state.resolve_uuid(partial_uuid)
    if not full_uuid:
        raise TopicNotFoundError(f"Téma '{partial_uuid}' nenalezeno.")
    data = state.load_topic(full_uuid)
    if data is None:
        raise TopicNotFoundError(f"Nelze načíst téma '{full_uuid}'.")
    return full_uuid, data


# ── topic lifecycle ──────────────────────────────────────────────────────────


def init(name, notes=None):
    """Create a new tracked topic. Returns (uuid, topic_data)."""
    topic_uuid = str(uuid_mod.uuid4())
    data = {
        "uuid": topic_uuid,
        "name": name,
        "notes": [notes] if notes else [],
        "areas": [],
        "created_at": now_utc(),
        "last_check_at": None,
    }
    state.save_topic(topic_uuid, data)
    return topic_uuid, data


def show(partial_uuid):
    """Return (full_uuid, topic, report_summaries)."""
    full_uuid, data = _require_topic(partial_uuid)
    return full_uuid, data, state.report_summaries(full_uuid)


def delete(partial_uuid):
    """Delete a topic and best-effort-delete its automation.

    Atomic: rename → rmtree, so an interrupted delete never leaves an orphan
    topic dir visible to list/show.
    """
    full_uuid, data = _require_topic(partial_uuid)
    automation_id = data.get("automation_id")
    if automation_id:
        try:
            subprocess.run(
                [CDXCTL_BIN, "automation", "delete", automation_id],
                capture_output=True, text=True, timeout=15,
            )
        except Exception:
            pass  # non-fatal — automation may already be gone

    target = state.topic_dir(full_uuid)
    trash = f"{target}.deleted-{uuid_mod.uuid4().hex}"
    os.rename(target, trash)
    shutil.rmtree(trash, ignore_errors=True)
    return full_uuid


def touch(partial_uuid):
    """Update last_check_at to now, without creating a report."""
    full_uuid, data = _require_topic(partial_uuid)
    data["last_check_at"] = now_utc()
    state.save_topic(full_uuid, data)
    return full_uuid, data["last_check_at"]


def set_automation(partial_uuid, automation_id):
    """Store automation ID on the topic so delete() can tear it down."""
    full_uuid, data = _require_topic(partial_uuid)
    data["automation_id"] = automation_id
    state.save_topic(full_uuid, data)
    return full_uuid


def list_all():
    """Return list of dicts summarizing each tracked topic, sorted by activity.

    Order: topics with unconfirmed reports first, then topics with confirmed
    reports (most recent on top), then ones without reports. Within each tier
    we use the most recent report date; created_at breaks final ties.
    """
    out = []
    for t_uuid, data in state.all_topics():
        summaries = state.report_summaries(t_uuid)
        unconfirmed = sum(1 for s in summaries if not s.get("confirmed_on"))
        out.append({
            "uuid": t_uuid,
            "name": data.get("name", ""),
            "notes": data.get("notes", []),
            "areas": data.get("areas", []),
            "created_at": data.get("created_at"),
            "last_check_at": data.get("last_check_at"),
            "automation_id": data.get("automation_id"),
            "total_reports": len(summaries),
            "unconfirmed_reports": unconfirmed,
            "_report_summaries": summaries,  # stripped before returning
        })

    def sort_key(t):
        summaries = t["_report_summaries"]
        unconfirmed = [s for s in summaries if not s.get("confirmed_on")]
        latest_unconfirmed = max(
            (s.get("checked_at") or s.get("report_id", "") for s in unconfirmed),
            default="",
        )
        latest_confirmed = max(
            (
                s.get("confirmed_on") or s.get("checked_at") or s.get("report_id", "")
                for s in summaries if s.get("confirmed_on")
            ),
            default="",
        )
        tier = 2 if unconfirmed else (1 if summaries else 0)
        activity = latest_unconfirmed if unconfirmed else latest_confirmed
        return (tier, activity, t.get("created_at") or "")

    out.sort(key=sort_key, reverse=True)
    for t in out:
        t.pop("_report_summaries", None)
    return out


# ── notes ────────────────────────────────────────────────────────────────────


def note_add(partial_uuid, text):
    full_uuid, data = _require_topic(partial_uuid)
    notes = data.get("notes", [])
    notes.append(text)
    data["notes"] = notes
    state.save_topic(full_uuid, data)
    return len(notes) - 1


def note_list(partial_uuid):
    _, data = _require_topic(partial_uuid)
    return data.get("notes", [])


def note_remove(partial_uuid, index):
    full_uuid, data = _require_topic(partial_uuid)
    notes = data.get("notes", [])
    if index < 0 or index >= len(notes):
        raise NoteIndexError(
            f"Index {index} mimo rozsah (0..{len(notes) - 1})."
        )
    removed = notes.pop(index)
    data["notes"] = notes
    state.save_topic(full_uuid, data)
    return removed


# ── areas ────────────────────────────────────────────────────────────────────


def area_add(partial_uuid, name):
    full_uuid, data = _require_topic(partial_uuid)
    areas = data.get("areas", [])
    if any(a.get("name", "").lower() == name.lower() for a in areas):
        raise AreaAlreadyExistsError(f"Oblast '{name}' již existuje.")
    areas.append({
        "name": name,
        "baseline_summary": None,
        "added_at": now_utc(),
    })
    data["areas"] = areas
    state.save_topic(full_uuid, data)
    return len(areas) - 1


def area_list(partial_uuid):
    _, data = _require_topic(partial_uuid)
    return data.get("areas", [])


def area_remove(partial_uuid, index):
    full_uuid, data = _require_topic(partial_uuid)
    areas = data.get("areas", [])
    if index < 0 or index >= len(areas):
        raise AreaIndexError(
            f"Index {index} mimo rozsah (0..{len(areas) - 1})."
        )
    removed = areas.pop(index)
    data["areas"] = areas
    state.save_topic(full_uuid, data)
    return removed


def area_set_baseline(partial_uuid, index, summary):
    full_uuid, data = _require_topic(partial_uuid)
    areas = data.get("areas", [])
    if index < 0 or index >= len(areas):
        raise AreaIndexError(
            f"Index {index} mimo rozsah (0..{len(areas) - 1})."
        )
    areas[index]["baseline_summary"] = summary
    areas[index]["baseline_set_at"] = now_utc()
    data["areas"] = areas
    state.save_topic(full_uuid, data)
    return areas[index]


# ── reports ──────────────────────────────────────────────────────────────────


def save_report(partial_uuid, file_path=None, stream=None):
    """Save a report from JSON. Exactly one of `file_path` or `stream` is used;
    if both are None, reads from sys.stdin.

    Returns the saved report_id.

    Raises ReportSourceError on I/O / JSON errors.
    """
    full_uuid, data = _require_topic(partial_uuid)

    try:
        if file_path:
            with open(file_path, "r", encoding="utf-8") as f:
                report = json.load(f)
        else:
            report = json.load(stream if stream is not None else sys.stdin)
    except FileNotFoundError:
        raise ReportSourceError(f"Soubor '{file_path}' nenalezen.")
    except json.JSONDecodeError as e:
        source = f"soubor '{file_path}'" if file_path else "stdin"
        raise ReportSourceError(f"{source} neobsahuje validní JSON: {e}")
    except OSError as e:
        raise ReportSourceError(f"Nelze číst '{file_path}': {e}")

    if "checked_at" not in report:
        report["checked_at"] = now_utc()
    if "confirmed_on" not in report:
        report["confirmed_on"] = None

    report_id = report.get("report_id")
    if not report_id:
        report_id = now_utc()[:10]
        report["report_id"] = report_id

    state.save_report(full_uuid, report_id, report)
    data["last_check_at"] = report["checked_at"]
    state.save_topic(full_uuid, data)
    return report_id


def list_reports(partial_uuid):
    full_uuid, _ = _require_topic(partial_uuid)
    return state.report_summaries(full_uuid)


def show_report(partial_uuid, report_id):
    full_uuid, _ = _require_topic(partial_uuid)
    report = state.load_report(full_uuid, report_id)
    if report is None:
        raise ReportNotFoundError(f"Report '{report_id}' nenalezen.")
    return report


def confirm_report(partial_uuid, report_id):
    full_uuid, _ = _require_topic(partial_uuid)
    report = state.load_report(full_uuid, report_id)
    if report is None:
        raise ReportNotFoundError(f"Report '{report_id}' nenalezen.")
    report["confirmed_on"] = now_utc()
    state.save_report(full_uuid, report_id, report)
    return report["confirmed_on"]


# ── uuid helpers for handler ─────────────────────────────────────────────────


def find_topic(partial_uuid):
    """Best-effort lookup used by HTTP handlers: returns (full_uuid, data) or
    (None, None). Never raises."""
    try:
        full_uuid = state.resolve_uuid(partial_uuid)
    except Exception:
        return None, None
    if not full_uuid:
        return None, None
    data = state.load_topic(full_uuid)
    if data is None:
        return None, None
    return full_uuid, data
