"""Verbal (word-mark) similarity scoring — pure standard library, no deps.

The goal is to approximate the *aural + visual* similarity of two trademark
verbal elements, which (together with conceptual similarity and similarity of
goods/services) drives the legal "likelihood of confusion" test. We deliberately
combine several complementary orthographic signals plus a Czech-aware phonetic
key, because no single string metric captures trademark confusability well:

  - Jaro–Winkler   → rewards a shared prefix (brand beginnings matter most)
  - Levenshtein    → overall edit proximity
  - trigram Jaccard→ shared character chunks, order-tolerant
  - phonetic key   → "sounds like" matches across spelling variants

All scores are in [0, 1]; 1.0 means identical after normalisation.
"""

import re
import unicodedata

__all__ = [
    "normalize",
    "jaro_winkler",
    "levenshtein_ratio",
    "trigram_jaccard",
    "czech_phonetic_key",
    "score",
]


# ── normalisation ─────────────────────────────────────────────────────────────


def _strip_accents(value: str) -> str:
    decomposed = unicodedata.normalize("NFKD", value or "")
    return "".join(c for c in decomposed if not unicodedata.combining(c))


def normalize(value: str) -> str:
    """Casefold, drop diacritics, collapse non-alphanumerics into single spaces."""
    folded = _strip_accents(value or "").lower()
    folded = re.sub(r"[^0-9a-z]+", " ", folded)
    return re.sub(r"\s+", " ", folded).strip()


# ── orthographic metrics ───────────────────────────────────────────────────────


def jaro(a: str, b: str) -> float:
    if a == b:
        return 1.0
    if not a or not b:
        return 0.0
    match_distance = max(len(a), len(b)) // 2 - 1
    if match_distance < 0:
        match_distance = 0
    a_matches = [False] * len(a)
    b_matches = [False] * len(b)
    matches = 0
    for i, ch in enumerate(a):
        lo = max(0, i - match_distance)
        hi = min(i + match_distance + 1, len(b))
        for j in range(lo, hi):
            if not b_matches[j] and b[j] == ch:
                a_matches[i] = b_matches[j] = True
                matches += 1
                break
    if matches == 0:
        return 0.0
    transpositions = 0
    k = 0
    for i in range(len(a)):
        if a_matches[i]:
            while not b_matches[k]:
                k += 1
            if a[i] != b[k]:
                transpositions += 1
            k += 1
    transpositions //= 2
    return (
        matches / len(a)
        + matches / len(b)
        + (matches - transpositions) / matches
    ) / 3.0


def jaro_winkler(a: str, b: str, prefix_weight: float = 0.1) -> float:
    j = jaro(a, b)
    prefix = 0
    for ca, cb in zip(a, b):
        if ca == cb:
            prefix += 1
        else:
            break
        if prefix == 4:
            break
    return j + prefix * prefix_weight * (1 - j)


def levenshtein(a: str, b: str) -> int:
    if a == b:
        return 0
    if not a:
        return len(b)
    if not b:
        return len(a)
    previous = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        current = [i]
        for j, cb in enumerate(b, 1):
            current.append(
                min(
                    previous[j] + 1,
                    current[j - 1] + 1,
                    previous[j - 1] + (ca != cb),
                )
            )
        previous = current
    return previous[-1]


def levenshtein_ratio(a: str, b: str) -> float:
    longest = max(len(a), len(b))
    if longest == 0:
        return 1.0
    return 1.0 - levenshtein(a, b) / longest


def _trigrams(value: str) -> set:
    padded = f"  {value} "
    return {padded[i : i + 3] for i in range(len(padded) - 2)}


def trigram_jaccard(a: str, b: str) -> float:
    if not a and not b:
        return 1.0
    ta, tb = _trigrams(a), _trigrams(b)
    if not ta or not tb:
        return 0.0
    inter = len(ta & tb)
    union = len(ta | tb)
    return inter / union if union else 0.0


# ── Czech-aware phonetic key ────────────────────────────────────────────────────

# Ordered (multi-char first) substitutions collapsing spelling onto a rough
# pronunciation skeleton. Applied to the accent-stripped, lower-cased string.
_PHONETIC_RULES = [
    ("sch", "s"),
    ("ch", "x"),
    ("ck", "k"),
    ("dž", "z"),
    ("ph", "f"),
    ("th", "t"),
    ("qu", "kv"),
    ("ou", "o"),
    ("au", "o"),
    ("ei", "aj"),
    ("ie", "i"),
]
_PHONETIC_CHAR = str.maketrans(
    {
        "w": "v",
        "q": "k",
        "x": "ks",
        "y": "i",
        "g": "k",
        "d": "t",
        "b": "p",
        "z": "s",
        "c": "k",
    }
)


def czech_phonetic_key(value: str) -> str:
    """A coarse Czech/English pronunciation skeleton for aural comparison.

    Maps voiced/voiceless pairs together, normalises common digraphs and
    foreign spellings, then drops repeated letters. Not a strict phonetic
    transcription — just enough to make "Kodexis"≈"Codexis", "Fillip"≈"Philip".
    """
    text = _strip_accents(value or "").lower()
    text = re.sub(r"[^a-z]+", "", text)
    for src, dst in _PHONETIC_RULES:
        text = text.replace(src, dst)
    text = text.translate(_PHONETIC_CHAR)
    # collapse runs of the same letter ("ss" → "s")
    text = re.sub(r"(.)\1+", r"\1", text)
    return text


# ── combined score ───────────────────────────────────────────────────────────


def _orthographic(a: str, b: str) -> float:
    if not a or not b:
        return 0.0
    jw = jaro_winkler(a, b)
    lev = levenshtein_ratio(a, b)
    tri = trigram_jaccard(a, b)
    base = 0.45 * jw + 0.30 * lev + 0.25 * tri
    # Containment boost: a short distinctive mark fully inside a longer one is a
    # strong confusion signal (e.g. "CODEX" inside "CODEXIS PLUS").
    if a in b or b in a:
        base = max(base, 0.85)
    return min(1.0, base)


def score(a_text: str, b_text: str) -> dict:
    """Return {orthographic, phonetic, combined} similarity in [0,1] for two marks."""
    na, nb = normalize(a_text), normalize(b_text)
    if not na or not nb:
        return {"orthographic": 0.0, "phonetic": 0.0, "combined": 0.0}
    if na == nb:
        return {"orthographic": 1.0, "phonetic": 1.0, "combined": 1.0}

    orthographic = _orthographic(na, nb)
    pa, pb = czech_phonetic_key(a_text), czech_phonetic_key(b_text)
    phonetic = _orthographic(pa, pb) if pa and pb else 0.0

    combined = 0.6 * orthographic + 0.4 * phonetic
    # First-token agreement (shared dominant word) lifts the score.
    if na.split(" ")[0] == nb.split(" ")[0]:
        combined = max(combined, 0.75)
    return {
        "orthographic": round(orthographic, 4),
        "phonetic": round(phonetic, 4),
        "combined": round(min(1.0, combined), 4),
    }
