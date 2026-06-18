"""Deterministic legislation-change check for watched folders.

Reads each folder's ``.watched/watched.json``, collects the distinct referenced
acts (those that resolved to a CODEXIS ``codexisId``) and compares the current
CODEXIS version id against the stored baseline in ``.watched/tracking-state.json``.

NO AI is involved — change detection is a version-id comparison. Reuses the same
CODEXIS clients and compare-URL helpers as the by-id document tracker.
"""

import os

from . import clients, diff, folders, notify
from .state import now_utc


def _seed_legislation(tracking, ref):
    """Ensure a tracking entry exists for a referenced act and refresh its label.

    Unresolved references (no ``codexisId``) are skipped — they are listed in
    watched.json for humans but cannot be tracked deterministically.
    """
    codexis_id = ref.get("codexisId")
    if not codexis_id:
        return
    leg = tracking["legislation"]
    if codexis_id in leg:
        if ref.get("uri"):
            leg[codexis_id]["uri"] = ref["uri"]
        if ref.get("text"):
            leg[codexis_id]["text"] = ref["text"]
        return
    try:
        versions = clients.get_versions(codexis_id)
    except clients.CdxClientError:
        return
    latest = clients.get_latest_version_id(versions)
    if not latest:
        return
    leg[codexis_id] = {
        "uri": ref.get("uri", ""),
        "text": ref.get("text", ""),
        "baselineVersionId": latest,
        "lastKnownVersionId": latest,
        "lastCheckAt": now_utc(),
        "changes": [],
    }


def check_folder(root, printer=None):
    """Check one folder. Returns {root, name, changes:[...], errors:[...]}."""
    def _log(msg):
        if printer:
            printer(msg)

    abs_root = os.path.abspath(os.path.expanduser(str(root)))
    entry = folders.find_in_index(abs_root) or {}
    name = entry.get("name") or os.path.basename(abs_root.rstrip(os.sep)) or abs_root

    tracking = folders.read_tracking(abs_root) or folders.empty_tracking()

    for ref in folders.unique_legislation(abs_root):
        _seed_legislation(tracking, ref)

    new_changes = []
    errors = []
    for codexis_id, leg in tracking.get("legislation", {}).items():
        baseline = leg.get("baselineVersionId")
        label = leg.get("text") or codexis_id
        try:
            versions = clients.get_versions(codexis_id)
        except clients.CdxClientError as exc:
            errors.append(f"{label}: {exc}")
            _log(f"  {label}: {exc}")
            continue
        latest = clients.get_latest_version_id(versions)
        if not latest:
            errors.append(f"{label}: nepodařilo se získat verze")
            continue

        leg["lastKnownVersionId"] = latest
        leg["lastCheckAt"] = now_utc()

        if latest == baseline:
            _log(f"  {label}: beze změn ({latest})")
            continue
        already = any(
            c.get("old_version_id") == baseline and c.get("new_version_id") == latest
            for c in leg.get("changes", [])
        )
        if already:
            continue

        _log(f"  {label}: NOVÁ VERZE {latest} (baseline {baseline})")
        version_info = clients.find_version_info(versions, latest)
        change = {
            "detected_on": now_utc(),
            "effective_on": version_info.get("validFrom") if version_info else None,
            "old_version_id": baseline,
            "new_version_id": latest,
            "amendments": clients.resolve_amendments(versions, latest),
            "compare_url": diff.build_compare_url(baseline, latest),
            "confirmed_on": None,
        }
        leg.setdefault("changes", []).append(change)
        new_changes.append({
            "codexisId": codexis_id,
            "uri": leg.get("uri"),
            "text": leg.get("text"),
            **change,
        })

    tracking["lastCheckAt"] = now_utc()
    folders.write_tracking(abs_root, tracking)
    return {"root": abs_root, "name": name, "changes": new_changes, "errors": errors}


def check_folders(root=None, printer=None, send_notifications=True):
    """Check one or all watched folders, then notify on detected changes.

    Returns {checked, changes_found, errors, results}.
    """
    if root:
        roots = [os.path.abspath(os.path.expanduser(str(root)))]
    else:
        roots = [e.get("root") for e in folders.load_index() if e.get("root")]

    results = []
    errors = []
    changes_found = 0
    for r in roots:
        res = check_folder(r, printer=printer)
        results.append(res)
        changes_found += len(res["changes"])
        errors.extend(res["errors"])

    if send_notifications and changes_found:
        try:
            notify.notify_changes(results)
        except Exception as exc:  # notifications must never fail the run
            errors.append(f"notifikace: {exc}")

    return {
        "checked": len(roots),
        "changes_found": changes_found,
        "errors": errors,
        "results": results,
    }


def confirm_folder(root, codexis_id=None):
    """Mark detected changes as read and advance the baseline(s). Returns count."""
    abs_root = os.path.abspath(os.path.expanduser(str(root)))
    tracking = folders.read_tracking(abs_root)
    if not tracking:
        return 0
    now = now_utc()
    items = tracking.get("legislation", {})
    targets = [codexis_id] if codexis_id else list(items.keys())
    marked = 0
    for cid in targets:
        leg = items.get(cid)
        if not leg:
            continue
        unconfirmed = [c for c in leg.get("changes", []) if not c.get("confirmed_on")]
        for change in unconfirmed:
            change["confirmed_on"] = now
            marked += 1
        if unconfirmed:
            leg["baselineVersionId"] = leg.get("lastKnownVersionId", leg.get("baselineVersionId"))
    if marked:
        folders.write_tracking(abs_root, tracking)
    return marked
