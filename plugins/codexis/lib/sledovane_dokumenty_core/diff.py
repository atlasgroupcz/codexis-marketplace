"""Text normalization and version-to-version diff generation.

Two representations of changes:
- compute_version_diff(...) → unified diff per changed section (for storage / UI)
- build_change_text(...)    → word-level excerpts with context for the LLM prompt
"""

import difflib
import re

from . import clients


# ── text normalization ───────────────────────────────────────────────────────


def strip_changes_in_time(text):
    """Remove <changes_in_time>...</changes_in_time> blocks (authorial metadata)."""
    return re.sub(
        r"<changes_in_time>.*?</changes_in_time>", "", text, flags=re.DOTALL
    ).strip()


def normalize_cdx_links(text):
    """Drop the date suffix from cdx://doc/XXX links so link-only version bumps
    don't register as content changes."""
    return re.sub(r"(cdx://doc/[A-Z]+\d+)_\d{4}_\d{2}_\d{2}", r"\1", text)


def strip_part_markers(text):
    """Remove [?part=...] marker lines and resolve markdown links to plain text."""
    text = re.sub(r"^\s*\[\?part=[^\]]*\]\s*$", "", text, flags=re.MULTILINE)
    text = re.sub(r"\[([^\]]+?)\]\(cdx://[^)]*\)", r"\1", text)
    text = re.sub(r"\s*\(anchor in this document\)", "", text)
    return text


def normalize_text(text):
    """Apply all text cleanups in order."""
    return strip_part_markers(normalize_cdx_links(strip_changes_in_time(text)))


# ── section extraction ──────────────────────────────────────────────────────


def extract_section(text, element_id):
    """Return the slice of `text` belonging to the given [?part=element_id] section."""
    lines = text.split("\n")
    capture = False
    result = []
    marker = f"[?part={element_id}]"
    for line in lines:
        if line.strip() == marker:
            capture = True
            continue
        if capture:
            if line.strip().startswith("[?part="):
                break
            result.append(line)
    return "\n".join(result).strip()


def extract_part_ids(text):
    """Return list of all [?part=...] element IDs present in `text`."""
    return re.findall(r"^\s*\[\?part=([^\]]+)\]", text, re.MULTILINE)


# ── diff algorithms ──────────────────────────────────────────────────────────


def unified_text_diff(old_text, new_text, label=""):
    """Unified diff between two plain-text blobs."""
    old_lines = old_text.splitlines(keepends=True)
    new_lines = new_text.splitlines(keepends=True)
    diff = difflib.unified_diff(
        old_lines, new_lines,
        fromfile=f"old {label}".strip(),
        tofile=f"new {label}".strip(),
        lineterm="",
    )
    return "".join(diff).strip()


def word_level_changes(old_text, new_text, context_words=10):
    """Return word-level changes with surrounding context, for LLM consumption."""
    old_words = old_text.split()
    new_words = new_text.split()
    sm = difflib.SequenceMatcher(None, old_words, new_words)
    changes = []
    for op, i1, i2, j1, j2 in sm.get_opcodes():
        if op == "equal":
            continue
        ctx_start_old = max(0, i1 - context_words)
        ctx_start_new = max(0, j1 - context_words)
        before = " ".join(old_words[ctx_start_old:i1])
        ctx_end_old = min(len(old_words), i2 + context_words)
        ctx_end_new = min(len(new_words), j2 + context_words)
        after_old = " ".join(old_words[i2:ctx_end_old])
        after_new = " ".join(new_words[j2:ctx_end_new])
        after = after_old or after_new
        removed = " ".join(old_words[i1:i2])
        added = " ".join(new_words[j1:j2])
        if op == "replace":
            changes.append(f"...{before} [-{removed}-] [+{added}+] {after}...")
        elif op == "delete":
            changes.append(f"...{before} [-{removed}-] {after}...")
        elif op == "insert":
            changes.append(f"...{before} [+{added}+] {after}...")
    return "\n\n".join(changes)


def per_section_changes(old_text, new_text):
    """Full old+new text for each section that changed. Returns None if the doc
    has no [?part=...] markers or no sections actually changed."""
    part_ids = extract_part_ids(new_text)
    if not part_ids:
        return None
    section_changes = []
    for pid in part_ids:
        old_s = normalize_text(extract_section(old_text, pid))
        new_s = normalize_text(extract_section(new_text, pid))
        if old_s == new_s:
            continue
        section_changes.append(
            f"### {pid}\nSTARÉ ZNĚNÍ:\n{old_s}\n\nNOVÉ ZNĚNÍ:\n{new_s}"
        )
    if not section_changes:
        return None
    return "\n\n".join(section_changes)


# ── high-level orchestration ─────────────────────────────────────────────────


def compute_version_diff(baseline_vid, latest_vid, parts, printer=None):
    """Fetch texts and compute diff between two versions.

    Returns list of {"part": str, "diff": str}. `parts` restricts to given
    [?part=...] ids; empty means diff the whole document.

    `printer` is an optional callable for progress reporting (e.g. print).
    """
    def _log(msg):
        if printer:
            printer(msg)

    diff_parts = []
    if parts:
        for part_id in parts:
            try:
                old_text = clients.cdx_get_text(
                    f"cdx://doc/{baseline_vid}/text?part={part_id}"
                )
                new_text = clients.cdx_get_text(
                    f"cdx://doc/{latest_vid}/text?part={part_id}"
                )
            except clients.CdxClientError:
                continue
            old_clean = normalize_text(old_text)
            new_clean = normalize_text(new_text)
            if old_clean != new_clean:
                diff_parts.append({
                    "part": part_id,
                    "diff": unified_text_diff(old_clean, new_clean, label=part_id),
                })
                _log(f"    {part_id}: změněn")
            else:
                _log(f"    {part_id}: beze změn")
        return diff_parts

    try:
        old_text = clients.cdx_get_text(f"cdx://doc/{baseline_vid}/text")
        new_text = clients.cdx_get_text(f"cdx://doc/{latest_vid}/text")
    except clients.CdxClientError:
        return diff_parts
    old_clean = normalize_text(old_text)
    new_clean = normalize_text(new_text)
    if old_clean != new_clean:
        diff_parts.append({
            "part": "full",
            "diff": unified_text_diff(old_clean, new_clean),
        })
    return diff_parts


def build_change_text(baseline_vid, latest_vid, parts):
    """Build the diff text that gets fed to the LLM. Returns None if no changes
    or texts could not be fetched."""
    if parts:
        all_old = []
        all_new = []
        for part_id in parts:
            try:
                old_text = clients.cdx_get_text(
                    f"cdx://doc/{baseline_vid}/text?part={part_id}"
                )
                new_text = clients.cdx_get_text(
                    f"cdx://doc/{latest_vid}/text?part={part_id}"
                )
            except clients.CdxClientError:
                return None
            all_old.append(normalize_text(old_text))
            all_new.append(normalize_text(new_text))
        return word_level_changes(
            "\n\n".join(all_old), "\n\n".join(all_new), context_words=30
        )

    try:
        old_text = clients.cdx_get_text(f"cdx://doc/{baseline_vid}/text")
        new_text = clients.cdx_get_text(f"cdx://doc/{latest_vid}/text")
    except clients.CdxClientError:
        return None

    changes = per_section_changes(old_text, new_text)
    if changes:
        return changes
    return word_level_changes(
        normalize_text(old_text), normalize_text(new_text), context_words=30
    )


def build_summary_prompt(doc_name, user_notes):
    """Build the LLM prompt used by generate_summary."""
    prompt = (
        f"Shrň věcné změny v předpisu '{doc_name}'. "
        "Vstup obsahuje sekce se STARÝM a NOVÝM ZNĚNÍM. "
        "Popiš POUZE skutečné změny v textu zákona. "
        "Přeskoč sekce kde se změnily jen poznámky autora, čísla vyhlášek nebo sdělení. "
        "Odpověz česky, stručně, plynulým textem."
    )
    if user_notes:
        notes_text = "; ".join(user_notes)
        prompt += (
            f"\n\nUživatele zvláště zajímá: {notes_text}. "
            "Pokud se změny týkají těchto témat, zdůrazni to."
        )
    return prompt


def generate_summary(doc_name, baseline_vid, latest_vid, parts, user_notes):
    """Return AI summary for a version change, or None if not available."""
    change_text = build_change_text(baseline_vid, latest_vid, parts)
    if not change_text:
        return None
    return clients.llm_extract(change_text, build_summary_prompt(doc_name, user_notes))


def build_compare_url(baseline_vid, latest_vid):
    """CODEXIS web UI comparison link."""
    return (
        f"https://next.codexis.cz/porovnat"
        f"?sourceDocId={baseline_vid}"
        f"&targetDocId={latest_vid}"
        f"&puvodniZneni={baseline_vid}"
        f"&viewType=INSIDE"
        f"&changesOnly=true"
    )
