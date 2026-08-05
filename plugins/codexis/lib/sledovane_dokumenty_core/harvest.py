"""One-pass AI harvest of legislative references from watched documents.

For every document that needs (re)harvesting, we send the file to the daemon
extract endpoint (``/rest/v1/plugin/llm/extract``) ONCE, asking for the
legislation it references. The model's answer is then turned into stable
contract entries: a human ``text`` title, a canonical ``uri`` and - when the
reference resolves to a Czech act in CODEXIS - a ``codexisId`` that the
deterministic runner uses for change tracking.

AI is used here and ONLY here. The daily check never calls a model.
"""

import json
import re

from . import clients, folders

# Object root (strict structured output needs an object, not a bare array).
HARVEST_SCHEMA = json.dumps(
    {
        "type": "object",
        "additionalProperties": False,
        "properties": {
            "references": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": False,
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "Human-friendly title incl. the relevant part, e.g. "
                            "'Zákon č. 89/2012 Sb., občanský zákoník - § 2079 a násl.'",
                        },
                        "citation": {
                            "type": "string",
                            "description": "Canonical citation if identifiable, e.g. '89/2012 Sb.' "
                            "or 'Nařízení (EU) 2016/679'. Empty string if unknown.",
                        },
                        "uri": {
                            "type": "string",
                            "description": "Canonical public URL of the legislation if known, else "
                            "empty string.",
                        },
                    },
                    "required": ["text", "citation", "uri"],
                },
            },
        },
        "required": ["references"],
    }
)

HARVEST_QUERY = (
    "Projdi tento dokument a vypiš VŠECHNY odkazy na právní předpisy "
    "(zákony, vyhlášky, nařízení, novely, předpisy EU), na které dokument "
    "odkazuje nebo z nichž vychází. Pro každý uveď čtivý název (text), "
    "kanonickou citaci (např. '89/2012 Sb.') pokud ji lze určit, a kanonickou "
    "veřejnou URL pokud ji znáš. Nevypisuj judikaturu ani smluvní ujednání - "
    "pouze odkazy na legislativu. Vrať POUZE JSON dle schématu."
)

# A year as legislation cites it. Written once: the Czech act pattern embeds it and
# the EU pair disambiguation tests against it, and the two used to disagree.
_YEAR = r"(?:19|20)\d{2}"
# 89/2012, 262/2006 Sb. … - a Czech act number followed by a year.
_CZ_ACT_RE = re.compile(rf"(\d{{1,4}})\s*/\s*({_YEAR})\b")
_YEAR_RE = re.compile(rf"^{_YEAR}$")

# CELEX sector-3 type letter per EU act type, as the type is written in a Czech
# citation. This is the single vocabulary of EU act types: the markers below that
# keep an EU citation out of the Czech resolver derive from it, so adding a type
# here teaches both sides at once.
_EU_TYPE_LETTERS = {"nařízení": "R", "směrnice": "L", "rozhodnutí": "D"}
# An EU act may also be cited by treaty abbreviation alone, with no type word.
_EU_MARKERS = (*_EU_TYPE_LETTERS, "eu", "es", "ehs")
# Either NUMBER/YEAR (pre-2015 acts) or YEAR/NUMBER (2015 on).
_EU_NUMBER_RE = re.compile(r"(\d{1,4})\s*/\s*(\d{1,4})")


def _looks_like_year(value):
    return _YEAR_RE.match(value) is not None


def parse_eu_citation(citation, text=""):
    """Return ``(type_letter, year, number)`` for an EU act citation, else None.

    These are the coordinates the citation carries; CODEXIS composes the CELEX
    from them. The one thing that has to be decided here is which side of the
    pair is the year: EU numbering flipped in 2015, so older acts are cited
    NUMBER/YEAR (``988/2009``) and newer ones YEAR/NUMBER (``2016/679``).
    """
    low = f"{citation} {text}".lower()
    letter = next(
        (letter for keyword, letter in _EU_TYPE_LETTERS.items() if keyword in low),
        None,
    )
    if not letter:
        return None

    m = _EU_NUMBER_RE.search(low)
    if not m:
        return None
    left, right = m.group(1), m.group(2)
    if _looks_like_year(right):
        year, number = right, left
    elif _looks_like_year(left):
        year, number = left, right
    else:
        return None
    # Unpadded, so "679" and "0679" key the resolver cache identically; CODEXIS
    # pads it when composing the CELEX.
    return letter, year, number.lstrip("0") or "0"


def canonical_eu_uri(celex):
    """Stable public URL for an EU act (EUR-Lex, the official EU portal)."""
    return f"https://eur-lex.europa.eu/legal-content/CS/TXT/?uri=CELEX:{celex}"


def canonical_cz_uri(num, year):
    """Stable public URL for a Czech act (Zákony pro lidi)."""
    return f"https://www.zakonyprolidi.cz/cs/{year}-{num}"


def parse_cz_citation(citation, text=""):
    """Return (num, year) if the reference looks like a Czech act, else None.

    EU regulations/directives (which often also contain ``NNNN/NNN``) are skipped
    so they are not mis-resolved as Czech acts.
    """
    haystack = f"{citation} {text}"
    low = haystack.lower()
    if "sb." not in low and any(m.lower() in low for m in _EU_MARKERS):
        return None
    m = _CZ_ACT_RE.search(citation) or _CZ_ACT_RE.search(text)
    if not m:
        return None
    return m.group(1), m.group(2)


def build_legislation_entry(ref):
    """Turn a model reference {text, citation, uri} into a contract entry.

    Resolves Czech acts to a canonical Zákony-pro-lidi URI + CODEXIS base id, and
    EU acts to a CODEXIS id by their citation coordinates. Falls back to the
    model-supplied URI (or empty) for anything unresolved.
    """
    text = (ref.get("text") or ref.get("citation") or "").strip()
    citation = (ref.get("citation") or "").strip()
    model_uri = (ref.get("uri") or "").strip()

    entry = {"uri": model_uri, "text": text or citation or "Neznámý předpis"}

    parsed = parse_cz_citation(citation, text)
    if parsed:
        num, year = parsed
        entry["uri"] = canonical_cz_uri(num, year)
        codexis_id = clients.resolve_cz_law(num, year)
    else:
        eu_parts = parse_eu_citation(citation, text)
        resolved = clients.resolve_eu_law(*eu_parts) if eu_parts else None
        codexis_id, celex = resolved or (None, None)
        if celex:
            entry["uri"] = canonical_eu_uri(celex)

    if codexis_id:
        entry["codexisId"] = codexis_id
    return entry


def _parse_references(response):
    """Coerce the extract response into a list of reference dicts."""
    if response is None:
        return None
    if isinstance(response, str):
        try:
            response = json.loads(response)
        except json.JSONDecodeError:
            return None
    if isinstance(response, dict):
        refs = response.get("references", [])
    elif isinstance(response, list):
        refs = response
    else:
        return None
    return [r for r in refs if isinstance(r, dict)]


def harvest_document(root, rel_path, printer=None):
    """Harvest one document. Returns the number of references written, or None
    on extract failure (document left pending)."""
    abs_path = folders.to_abs(root, rel_path)
    response = clients.llm_extract_file(
        abs_path,
        HARVEST_QUERY,
        schema=HARVEST_SCHEMA,
        schema_name="legislation_references",
    )
    refs = _parse_references(response)
    if refs is None:
        if printer:
            printer(f"  {rel_path}: extrakce selhala, zkusím příště")
        return None

    legislation = [build_legislation_entry(r) for r in refs]
    legislation = [e for e in legislation if e.get("text")]
    folders.set_references(root, rel_path, legislation)
    if printer:
        printer(f"  {rel_path}: {len(legislation)} odkazů")
    return len(legislation)


def harvest_folder(root, printer=None):
    """Harvest every document that needs it. Returns a summary dict.

    {"documents": n, "references": m, "failed": k}
    """
    abs_root = folders.normalize_root(root)
    if folders.read_watched(abs_root) is None:
        folders.sync_watched(abs_root)

    pending = folders.docs_needing_harvest(abs_root)
    documents = references = failed = 0
    for rel_path in pending:
        count = harvest_document(abs_root, rel_path, printer=printer)
        if count is None:
            failed += 1
        else:
            documents += 1
            references += count
    return {"documents": documents, "references": references, "failed": failed}
