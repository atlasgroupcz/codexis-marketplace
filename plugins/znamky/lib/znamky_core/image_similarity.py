"""Figurative (logo) similarity — perceptual hashing + colour + Vienna codes.

Pillow + imagehash are *optional* runtime deps (installed best-effort by the
plugin's postInstall hook). When they are missing we degrade gracefully: the
image-pixel signals return None and the caller falls back to Vienna-code overlap
plus the AI vision pass (Claude compares the actual logos). This keeps the core
import-safe in any environment, including the test runner.
"""

import math

try:  # optional heavy-ish deps
    from PIL import Image  # type: ignore
    import imagehash  # type: ignore

    HAS_IMAGE_LIBS = True
except Exception:  # noqa: BLE001 — any import failure means "no pixel signals"
    HAS_IMAGE_LIBS = False

__all__ = [
    "HAS_IMAGE_LIBS",
    "perceptual_hash",
    "hash_similarity",
    "color_signature",
    "color_similarity",
    "vienna_overlap",
]

_PHASH_BITS = 64


# ── perceptual hashing ─────────────────────────────────────────────────────────


def perceptual_hash(path: str):
    """Return {'phash':hex,'dhash':hex} for an image file, or None if unavailable."""
    if not HAS_IMAGE_LIBS or not path:
        return None
    try:
        with Image.open(path) as img:
            rgb = img.convert("RGB")
            return {
                "phash": str(imagehash.phash(rgb, hash_size=8)),
                "dhash": str(imagehash.dhash(rgb, hash_size=8)),
            }
    except Exception:  # noqa: BLE001
        return None


def _hamming_hex(a: str, b: str) -> int:
    ia = int(a, 16)
    ib = int(b, 16)
    return bin(ia ^ ib).count("1")


def hash_similarity(a, b):
    """Similarity in [0,1] from two perceptual-hash dicts, or None if either missing."""
    if not a or not b:
        return None
    sims = []
    for key in ("phash", "dhash"):
        if a.get(key) and b.get(key):
            try:
                dist = _hamming_hex(a[key], b[key])
            except ValueError:
                continue
            sims.append(1.0 - dist / _PHASH_BITS)
    if not sims:
        return None
    # Favour the stronger of the two hashes — either alone is a real match.
    return round(max(sims) * 0.7 + (sum(sims) / len(sims)) * 0.3, 4)


# ── colour signature ───────────────────────────────────────────────────────────


def color_signature(path: str):
    """Coarse hue/saturation/value histogram as a normalised vector, or None."""
    if not HAS_IMAGE_LIBS or not path:
        return None
    try:
        with Image.open(path) as img:
            hsv = img.convert("RGB").resize((64, 64)).convert("HSV")
            pixels = list(hsv.getdata())
    except Exception:  # noqa: BLE001
        return None
    bins = [0.0] * (8 + 4 + 4)
    for h, s, v in pixels:
        bins[h * 8 // 256] += 1
        bins[8 + s * 4 // 256] += 1
        bins[12 + v * 4 // 256] += 1
    total = sum(bins) or 1.0
    return [x / total for x in bins]


def color_similarity(a, b):
    """Cosine similarity in [0,1] of two colour signatures, or None if missing."""
    if not a or not b or len(a) != len(b):
        return None
    dot = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(y * y for y in b))
    if na == 0 or nb == 0:
        return None
    return round(max(0.0, dot / (na * nb)), 4)


# ── Vienna Classification overlap (no image needed) ──────────────────────────────


def _vienna_parts(code: str):
    return tuple(p for p in str(code).replace(" ", "").split(".") if p)


def vienna_overlap(codes_a, codes_b):
    """Hierarchy-aware similarity of two Vienna figurative-code sets, in [0,1].

    Vienna codes are dotted hierarchies (category.division.section, e.g.
    "26.4.1"). Exact matches count fully; sharing only the category/division
    counts as partial credit — figuratively related even if not identical.
    """
    a = [c for c in (codes_a or []) if c]
    b = [c for c in (codes_b or []) if c]
    if not a or not b:
        return 0.0
    best_total = 0.0
    for ca in a:
        pa = _vienna_parts(ca)
        best = 0.0
        for cb in b:
            pb = _vienna_parts(cb)
            shared = 0
            for x, y in zip(pa, pb):
                if x == y:
                    shared += 1
                else:
                    break
            depth = max(len(pa), len(pb)) or 1
            best = max(best, shared / depth)
        best_total += best
    return round(best_total / len(a), 4)
