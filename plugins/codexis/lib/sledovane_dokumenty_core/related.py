"""Related-document tracking: baseline snapshots + diff detection.

Each tracked document can have several "relation types" (e.g. JUDIKATURA,
PROVADECI_PREDPIS). For each type we snapshot the set of related doc IDs
returned by CODEXIS and compare on every check — new / removed IDs become
`related_change` entries in the parent state.
"""

import os

from . import clients, state
from .exceptions import DocumentNotTrackedError
from .state import now_utc

# Default display names per type. CODEXIS API may return typeName directly.
RELATED_TYPE_NAMES = {
    "SOUVISEJICI_LEGISLATIVA_CR": "Související legislativa ČR",
    "PROVADECI_PREDPIS": "Prováděcí předpis",
    "PASIVNI_NOVELA": "Pasivní novela",
    "AKTIVNI_DEROGACE": "Aktivní derogace",
    "SOUVISEJICI_LEGISLATIVA_EU": "Související legislativa EU",
    "JUDIKATURA": "Judikatura",
    "ODBORNY_CLANEK": "Odborný článek",
    "KOMENTAROVE_DILO": "Komentářové dílo",
}


def get_type_name(relation_type, counts_data=None):
    if counts_data:
        for item in counts_data:
            if item.get("type") == relation_type:
                return item.get("typeName", relation_type)
    return RELATED_TYPE_NAMES.get(relation_type, relation_type)


def capture_baseline(codexis_id, relation_type):
    """Fetch current related IDs and store them as the baseline. Returns count."""
    doc_ids = clients.fetch_all_related_ids(codexis_id, relation_type)
    state.save_related_baseline(codexis_id, relation_type, {
        "type": relation_type,
        "captured_at": now_utc(),
        "total_count": len(doc_ids),
        "doc_ids": doc_ids,
    })
    return len(doc_ids)


def enable_tracking(codexis_id, relation_type):
    """Add a relation type to the document's tracking. Returns updated state."""
    s = state.load_state(codexis_id)
    if s is None:
        raise DocumentNotTrackedError(
            f"Dokument {codexis_id} není sledován."
        )

    rt = s.get("related_tracking") or {}
    existing = list(rt.get("types", [])) if rt.get("enabled") else []
    if relation_type in existing:
        return s  # already tracked, idempotent

    capture_baseline(codexis_id, relation_type)
    existing.append(relation_type)
    s["related_tracking"] = {
        "enabled": True,
        "types": existing,
        "last_check_at": rt.get("last_check_at", now_utc()),
    }
    state.save_state(codexis_id, s)
    return s


def disable_tracking(codexis_id, relation_type):
    """Remove a relation type from tracking. Returns updated state."""
    s = state.load_state(codexis_id)
    if s is None:
        raise DocumentNotTrackedError(
            f"Dokument {codexis_id} není sledován."
        )
    rt = s.get("related_tracking") or {}
    existing = list(rt.get("types", []))
    if relation_type not in existing:
        return s
    state.delete_related_baselines(codexis_id, [relation_type])
    remaining = [t for t in existing if t != relation_type]
    s["related_tracking"] = {
        "enabled": bool(remaining),
        "types": remaining,
        "last_check_at": rt.get("last_check_at"),
    }
    state.save_state(codexis_id, s)
    return s


def detect_changes(codexis_id, relation_type, printer=None):
    """Compare current CODEXIS state against stored baseline for one relation type.

    Returns a `related_change` dict ready to append to state.changes, or None if
    no change. On API failure returns None and logs via printer.
    """
    def _log(msg):
        if printer:
            printer(msg)

    baseline = state.load_related_baseline(codexis_id, relation_type)
    if baseline is None:
        return None
    baseline_ids = set(baseline.get("doc_ids", []))

    try:
        current_ids_list = clients.fetch_all_related_ids(codexis_id, relation_type)
    except clients.CdxClientError as e:
        _log(f"  nepodařilo se stáhnout related {relation_type}: {e}")
        return None
    current_ids = set(current_ids_list)

    added_ids = current_ids - baseline_ids
    removed_ids = baseline_ids - current_ids
    if not added_ids and not removed_ids:
        return None

    type_name = get_type_name(relation_type)

    # Fetch titles for added/removed (cap at 10 to avoid API burst).
    added_docs = [
        {"docId": did, "title": clients.fetch_doc_title(did)}
        for did in sorted(added_ids)[:10]
    ]
    if len(added_ids) > 10:
        added_docs.append({"docId": "...", "title": f"a dalších {len(added_ids) - 10}"})

    removed_docs = [
        {"docId": did, "title": clients.fetch_doc_title(did)}
        for did in sorted(removed_ids)[:10]
    ]
    if len(removed_ids) > 10:
        removed_docs.append(
            {"docId": "...", "title": f"a dalších {len(removed_ids) - 10}"}
        )

    codexis_base = os.environ.get("CODEXIS_BASE_URL", "https://next.codexis.cz")
    desc_lines = []
    if added_docs:
        desc_lines.append(f"**Přidáno {len(added_ids)}** dokumentů typu {type_name}:")
        for doc in added_docs:
            if doc["docId"] == "...":
                desc_lines.append(f"- {doc['title']}")
            else:
                desc_lines.append(
                    f"- [{doc['title']}]({codexis_base}/doc/{doc['docId']})"
                )
    if removed_docs:
        if desc_lines:
            desc_lines.append("")
        desc_lines.append(f"**Odebráno {len(removed_ids)}** dokumentů typu {type_name}:")
        for doc in removed_docs:
            if doc["docId"] == "...":
                desc_lines.append(f"- {doc['title']}")
            else:
                desc_lines.append(
                    f"- [{doc['title']}]({codexis_base}/doc/{doc['docId']})"
                )

    source_documents = [
        {"codexisId": f"cdx://doc/{doc['docId']}", "name": doc["title"]}
        for doc in added_docs if doc["docId"] != "..."
    ]

    # Update baseline to current so next check starts fresh.
    state.save_related_baseline(codexis_id, relation_type, {
        "type": relation_type,
        "captured_at": now_utc(),
        "total_count": len(current_ids_list),
        "doc_ids": current_ids_list,
    })

    return {
        "change_type": "related_change",
        "relation_type": relation_type,
        "relation_type_name": type_name,
        "added_docs": added_docs,
        "removed_docs": removed_docs,
        "source_documents": source_documents,
        "description_md": "\n".join(desc_lines),
        "detected_on": now_utc(),
        "confirmed_on": None,
    }
