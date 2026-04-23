"""High-level tracking operations: add / remove / list / check / confirm / show.

Also hosts groups + notes CRUD and the check-automation bootstrap, since those
are small conveniences that don't justify their own modules.
"""

import json
import os
import shutil
import subprocess
import uuid as uuid_mod

from . import clients, diff, related, state
from .exceptions import (
    DocumentAlreadyTrackedError,
    DocumentError,
    DocumentNotFoundError,
    DocumentNotTrackedError,
    GroupNotFoundError,
    LlmDaemonError,
)
from .state import now_utc

CDXCTL_BIN = "cdxctl"
CHECK_AUTOMATION_TITLE = "Sledované dokumenty – kontrola změn"
CHECK_AUTOMATION_CRON = "0 8 * * 1"
CHECK_AUTOMATION_COMMAND = "cdx-sledovane-dokumenty check"
CHECK_AUTOMATION_DESC = "Pravidelná kontrola sledovaných dokumentů v CODEXIS."


# ── add / remove / list / show ───────────────────────────────────────────────


def add(codexis_id, parts=None):
    """Start tracking a document. Returns the saved state dict.

    Raises DocumentAlreadyTrackedError / DocumentNotFoundError / CdxClientError.
    """
    if state.load_state(codexis_id) is not None:
        raise DocumentAlreadyTrackedError(f"{codexis_id} je už sledován.")

    meta = clients.get_meta(codexis_id)
    name = clients.get_doc_name(meta)
    if not name:
        raise DocumentNotFoundError(
            f"Dokument {codexis_id} nebyl nalezen v CODEXIS."
        )

    versions = clients.get_versions(codexis_id)
    latest_vid = clients.get_latest_version_id(versions)
    if not latest_vid:
        raise DocumentNotFoundError(
            f"Nepodařilo se získat verze pro {codexis_id}."
        )

    s = {
        "uuid": str(uuid_mod.uuid4()),
        "codexisId": codexis_id,
        "name": name,
        "parts": list(parts or []),
        "added_on": now_utc(),
        "baseline_version_id": latest_vid,
        "last_known_version_id": latest_vid,
        "last_check_at": now_utc(),
        "changes": [],
    }
    state.save_state(codexis_id, s)
    ensure_check_automation()
    return s


def remove(codexis_id):
    """Stop tracking a document. Rename-then-rmtree keeps an interrupted delete safe."""
    s = state.load_state(codexis_id)
    if s is None:
        raise DocumentNotTrackedError(f"{codexis_id} není sledován.")

    target = state.state_dir(codexis_id)
    trash = f"{target}.deleted-{uuid_mod.uuid4().hex}"
    os.rename(target, trash)
    shutil.rmtree(trash, ignore_errors=True)

    # Clean group memberships.
    groups = state.load_groups()
    changed = False
    for group in groups:
        members = group.get("members", [])
        if codexis_id in members:
            members.remove(codexis_id)
            changed = True
    if changed:
        state.save_groups(groups)
    return s


def list_all():
    """Return list of state dicts for all tracked documents, sorted by activity.

    Order: unconfirmed changes first, then documents with confirmed changes
    (newest activity on top), then idle ones. Within each tier, newer activity
    wins; added_on breaks final ties.
    """
    docs = [
        s for s in (state.load_state(cid) for cid in state.all_tracked_ids())
        if s is not None
    ]

    def sort_key(s):
        changes = s.get("changes", [])
        unconfirmed = [c for c in changes if not c.get("confirmed_on")]
        latest_unconfirmed = max(
            (c.get("detected_on", "") for c in unconfirmed), default=""
        )
        latest_confirmed = max(
            (c.get("confirmed_on", "") for c in changes if c.get("confirmed_on")),
            default="",
        )
        tier = 2 if unconfirmed else (1 if changes else 0)
        activity = latest_unconfirmed if unconfirmed else latest_confirmed
        return (tier, activity, s.get("added_on", ""))

    docs.sort(key=sort_key, reverse=True)
    return docs


def show(codexis_id):
    """Return full state dict. Raises DocumentNotTrackedError."""
    s = state.load_state(codexis_id)
    if s is None:
        raise DocumentNotTrackedError(f"{codexis_id} není sledován.")
    return s


# ── check (version + related changes) ────────────────────────────────────────


def check(codexis_id=None, printer=None):
    """Run check over one or all tracked documents.

    Returns {"checked": int, "changes_found": int, "errors": [str]}.
    Use `printer` to surface progress (e.g. print). Never raises on per-doc
    failures — those go into `errors`.
    """
    def _log(msg):
        if printer:
            printer(msg)

    if codexis_id:
        ids = [codexis_id]
        if state.load_state(codexis_id) is None:
            raise DocumentNotTrackedError(f"{codexis_id} není sledován.")
    else:
        ids = state.all_tracked_ids()

    errors = []
    changes_found = 0

    # Phase 1: new versions
    for cid in ids:
        s = state.load_state(cid)
        if s is None:
            continue
        name = s.get("name", cid)
        baseline_vid = s.get("baseline_version_id")

        try:
            versions = clients.get_versions(cid)
        except clients.CdxClientError as e:
            errors.append(f"{name}: {e}")
            _log(f"  {name}: {e}")
            continue
        latest_vid = clients.get_latest_version_id(versions)
        if not latest_vid:
            errors.append(f"{name}: nepodařilo se získat verze")
            _log(f"  {name}: nepodařilo se získat verze")
            continue

        s["last_check_at"] = now_utc()
        s["last_known_version_id"] = latest_vid

        if latest_vid == baseline_vid:
            state.save_state(cid, s)
            _log(f"  {name}: beze změn (verze {latest_vid})")
            continue

        changes = s.get("changes", [])
        already = any(
            c.get("old_version_id") == baseline_vid
            and c.get("new_version_id") == latest_vid
            for c in changes
        )
        if already:
            state.save_state(cid, s)
            _log(f"  {name}: změna již zaznamenána ({baseline_vid} -> {latest_vid})")
            continue

        _log(f"  {name}: NOVÁ VERZE {latest_vid} (baseline: {baseline_vid})")
        changes_found += 1

        version_info = clients.find_version_info(versions, latest_vid)
        changes.append({
            "detected_on": now_utc(),
            "effective_on": version_info.get("validFrom") if version_info else None,
            "new_version_id": latest_vid,
            "old_version_id": baseline_vid,
            "amendments": clients.resolve_amendments(versions, latest_vid),
            "description_md": "",
            "compare_url": diff.build_compare_url(baseline_vid, latest_vid),
            "diffs": diff.compute_version_diff(
                baseline_vid, latest_vid, s.get("parts", []), printer=printer
            ),
            "confirmed_on": None,
            "summary_pending": True,
        })
        s["changes"] = changes
        state.save_state(cid, s)

    # Phase 1.5: related changes
    for cid in ids:
        s = state.load_state(cid)
        if s is None:
            continue
        rt = s.get("related_tracking") or {}
        if not rt.get("enabled"):
            continue
        name = s.get("name", cid)
        changes = s.get("changes", [])
        for rtype in rt.get("types", []):
            try:
                change = related.detect_changes(cid, rtype, printer=printer)
            except clients.CdxClientError as e:
                errors.append(f"{name}: related {rtype}: {e}")
                _log(f"  {name}: {e}")
                continue
            if not change:
                continue
            changes_found += 1
            _log(
                f"  {name}: ZMĚNA v {change['relation_type_name']} "
                f"(+{len(change['added_docs'])}, -{len(change['removed_docs'])})"
            )
            changes.append(change)
        rt["last_check_at"] = now_utc()
        s["related_tracking"] = rt
        s["changes"] = changes
        state.save_state(cid, s)

    # Phase 2: generate pending AI summaries
    summarize_pending(printer=printer)

    return {"checked": len(ids), "changes_found": changes_found, "errors": errors}


# ── confirm ──────────────────────────────────────────────────────────────────


def confirm(codexis_id, change_index=None):
    """Mark changes as read. Returns count marked.

    If `change_index` is given, only that change is marked and the baseline is
    NOT advanced. Without it, all unconfirmed changes are marked and the
    baseline advances to `last_known_version_id`.
    """
    s = state.load_state(codexis_id)
    if s is None:
        raise DocumentNotTrackedError(f"{codexis_id} není sledován.")
    changes = s.get("changes", [])
    timestamp = now_utc()

    if change_index is not None:
        if change_index < 0 or change_index >= len(changes):
            raise IndexError(
                f"Index {change_index} mimo rozsah (0..{len(changes) - 1})."
            )
        if changes[change_index].get("confirmed_on"):
            return 0
        changes[change_index]["confirmed_on"] = timestamp
        s["changes"] = changes
        state.save_state(codexis_id, s)
        return 1

    unconfirmed = [c for c in changes if not c.get("confirmed_on")]
    if not unconfirmed:
        return 0
    for c in unconfirmed:
        c["confirmed_on"] = timestamp
    s["baseline_version_id"] = s.get("last_known_version_id", s.get("baseline_version_id"))
    s["changes"] = changes
    state.save_state(codexis_id, s)
    return len(unconfirmed)


# ── AI summary backfill ──────────────────────────────────────────────────────


def summarize_pending(printer=None):
    """For every change with summary_pending=True, attempt to generate the summary."""
    def _log(msg):
        if printer:
            printer(msg)

    for cid in state.all_tracked_ids():
        s = state.load_state(cid)
        if s is None:
            continue
        name = s.get("name", cid)
        parts = s.get("parts", [])
        changed = False
        for change in s.get("changes", []):
            if not change.get("summary_pending"):
                continue
            old_vid = change.get("old_version_id")
            new_vid = change.get("new_version_id")
            if not old_vid or not new_vid:
                continue
            _log(f"  {name}: generating summary ({old_vid} -> {new_vid})")
            summary = diff.generate_summary(
                name, old_vid, new_vid, parts, s.get("user_notes", [])
            )
            if summary:
                change["description_md"] = summary
                change["summary_pending"] = False
                changed = True
                _log(f"  {name}: summary done")
        if changed:
            state.save_state(cid, s)


# ── groups ───────────────────────────────────────────────────────────────────


def group_add(codexis_id, group_name):
    """Add document to a group (create group if needed). Returns "added" | "already"."""
    s = state.load_state(codexis_id)
    if s is None:
        raise DocumentNotTrackedError(f"{codexis_id} není sledován.")

    groups = state.load_groups()
    idx, group = state.find_group(groups, group_name)
    if group is None:
        group = {"id": state.slugify(group_name), "name": group_name, "members": []}
        groups.append(group)
        idx = len(groups) - 1

    members = group.get("members", [])
    if codexis_id in members:
        return "already"
    members.append(codexis_id)
    group["members"] = members
    groups[idx] = group
    state.save_groups(groups)
    return "added"


def group_remove(codexis_id, group_name):
    groups = state.load_groups()
    idx, group = state.find_group(groups, group_name)
    if group is None:
        raise GroupNotFoundError(f"Skupina '{group_name}' neexistuje.")
    members = group.get("members", [])
    if codexis_id not in members:
        raise DocumentNotTrackedError(
            f"{codexis_id} není ve skupině '{group_name}'."
        )
    members.remove(codexis_id)
    group["members"] = members
    groups[idx] = group
    state.save_groups(groups)


def group_delete_by_id(group_id):
    """Delete a group by its slug id. No-op if not found."""
    groups = state.load_groups()
    remaining = [g for g in groups if g.get("id") != group_id]
    if len(remaining) != len(groups):
        state.save_groups(remaining)


def group_remove_by_id(codexis_id, group_id):
    """Remove document from a group by group slug id. No-op if not a member."""
    groups = state.load_groups()
    for g in groups:
        if g.get("id") == group_id:
            members = g.get("members", [])
            if codexis_id in members:
                members.remove(codexis_id)
                state.save_groups(groups)
            return
    # unknown group id → silent no-op to match FE expectations


def group_rename(group_id, new_name):
    """Rename a group, re-slugifying its id. Returns updated group or None."""
    groups = state.load_groups()
    for g in groups:
        if g.get("id") == group_id:
            g["name"] = new_name
            g["id"] = state.slugify(new_name)
            state.save_groups(groups)
            return g
    return None


def find_by_uuid(target_uuid):
    """Return (codexisId, state_dict) for the matching uuid, or (None, None)."""
    for cid in state.all_tracked_ids():
        s = state.load_state(cid)
        if s and s.get("uuid") == target_uuid:
            return cid, s
    return None, None


def group_delete(group_name):
    groups = state.load_groups()
    idx, group = state.find_group(groups, group_name)
    if group is None:
        raise GroupNotFoundError(f"Skupina '{group_name}' neexistuje.")
    groups.pop(idx)
    state.save_groups(groups)


def groups_list():
    return state.load_groups()


# ── notes ────────────────────────────────────────────────────────────────────


def note_add(codexis_id, text):
    s = state.load_state(codexis_id)
    if s is None:
        raise DocumentNotTrackedError(f"{codexis_id} není sledován.")
    notes = s.get("user_notes", [])
    notes.append(text)
    s["user_notes"] = notes
    state.save_state(codexis_id, s)


def note_list(codexis_id):
    s = state.load_state(codexis_id)
    if s is None:
        raise DocumentNotTrackedError(f"{codexis_id} není sledován.")
    return s.get("user_notes", [])


def note_remove(codexis_id, index):
    s = state.load_state(codexis_id)
    if s is None:
        raise DocumentNotTrackedError(f"{codexis_id} není sledován.")
    notes = s.get("user_notes", [])
    if index < 0 or index >= len(notes):
        raise IndexError(
            f"Index {index} mimo rozsah (0..{len(notes) - 1})."
        )
    removed = notes.pop(index)
    s["user_notes"] = notes
    state.save_state(codexis_id, s)
    return removed


# ── automation ───────────────────────────────────────────────────────────────


def ensure_check_automation():
    """Ensure the weekly check automation exists. Idempotent, non-fatal on failure."""
    try:
        result = subprocess.run(
            [CDXCTL_BIN, "automation", "list"],
            capture_output=True, text=True, timeout=15,
        )
        if result.returncode != 0:
            return
        automations = json.loads(result.stdout)
    except Exception:
        return

    if not isinstance(automations, list):
        return

    for a in automations:
        if (
            isinstance(a, dict)
            and a.get("type") == "COMMAND"
            and a.get("command") == CHECK_AUTOMATION_COMMAND
        ):
            return

    try:
        subprocess.run(
            [
                CDXCTL_BIN, "automation", "create-command",
                "--title", CHECK_AUTOMATION_TITLE,
                "--cron", CHECK_AUTOMATION_CRON,
                "--command", CHECK_AUTOMATION_COMMAND,
                "--description", CHECK_AUTOMATION_DESC,
            ],
            capture_output=True, text=True, timeout=15,
        )
    except Exception:
        pass
