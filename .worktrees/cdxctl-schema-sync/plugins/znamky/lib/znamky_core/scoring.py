"""Combine the per-signal similarities into one collision score + risk tier.

The weighting follows the legal "global appreciation" intuition: for a word
mark the verbal (aural/visual) similarity dominates; for a figurative mark the
image + Vienna-code similarity dominates; a combined mark blends both. The
overlap of goods/services classes (Nice) modulates the result — identical signs
in unrelated classes are a much weaker conflict than in the same class.

The deterministic score here is a *triage* signal that decides what surfaces to
the user and what the AI then assesses for likelihood of confusion. It is not
itself a legal conclusion.
"""

from . import image_similarity, text_similarity

__all__ = ["class_overlap", "score_candidate", "tier_for"]

HIGH_THRESHOLD = 0.78
MEDIUM_THRESHOLD = 0.6


def class_overlap(a_classes, b_classes) -> float:
    """Jaccard overlap of two Nice-class sets in [0,1] (empty sets → neutral 0.5)."""
    a = {int(c) for c in (a_classes or []) if str(c).strip().isdigit()}
    b = {int(c) for c in (b_classes or []) if str(c).strip().isdigit()}
    if not a or not b:
        return 0.5  # unknown classes: stay neutral, don't suppress the match
    inter = len(a & b)
    union = len(a | b)
    return round(inter / union, 4) if union else 0.0


def _weights(kind: str) -> dict:
    if kind == "figurative":
        return {"text": 0.15, "image": 0.85}
    if kind == "combined":
        return {"text": 0.55, "image": 0.45}
    return {"text": 1.0, "image": 0.0}  # word


def _image_component(watched, candidate) -> tuple:
    """Return (image_score_or_None, breakdown dict) for the figurative signals."""
    phash_sim = image_similarity.hash_similarity(
        watched.get("logo_phash"), candidate.get("logo_phash")
    )
    color_sim = image_similarity.color_similarity(
        watched.get("logo_color"), candidate.get("logo_color")
    )
    vienna = image_similarity.vienna_overlap(
        watched.get("vienna_codes"), candidate.get("vienna_codes")
    )
    breakdown = {
        "image_phash": phash_sim,
        "image_color": color_sim,
        "vienna_overlap": vienna,
    }
    pixel = [s for s in (phash_sim, color_sim) if s is not None]
    if pixel:
        # pixel hashes dominate; colour and Vienna refine.
        pixel_score = phash_sim if phash_sim is not None else 0.0
        if color_sim is not None:
            pixel_score = 0.8 * pixel_score + 0.2 * color_sim
        image_score = max(pixel_score, vienna * 0.9)
    elif vienna > 0:
        # No usable pixels (deps missing / no logo) — lean on Vienna codes.
        image_score = vienna * 0.8
    else:
        image_score = None
    return image_score, breakdown


def score_candidate(watched: dict, candidate: dict) -> dict:
    """Compute the full per-candidate score breakdown + overall + tier.

    `watched` is a tracked-mark state; `candidate` is a normalised source mark.
    Both may carry: text, nice_classes, vienna_codes, logo_phash, logo_color.
    """
    text = text_similarity.score(
        watched.get("text", ""), candidate.get("mark_text", "")
    )
    image_score, image_breakdown = _image_component(watched, candidate)
    classes = class_overlap(
        watched.get("nice_classes"), candidate.get("nice_classes")
    )

    weights = _weights(watched.get("kind", "word"))
    parts = []
    if weights["text"] and text["combined"] is not None:
        parts.append((weights["text"], text["combined"]))
    if weights["image"] and image_score is not None:
        parts.append((weights["image"], image_score))
    if not parts:
        sign_similarity = 0.0
    else:
        total_w = sum(w for w, _ in parts)
        sign_similarity = sum(w * v for w, v in parts) / total_w

    # Modulate by goods/services proximity: full weight when classes overlap,
    # damped (never zeroed) when they don't — confusion risk is lower but the
    # mark is still worth surfacing.
    class_factor = 0.55 + 0.45 * classes
    overall = round(min(1.0, sign_similarity * class_factor), 4)

    scores = {
        "text_orthographic": text["orthographic"],
        "text_phonetic": text["phonetic"],
        "text_combined": text["combined"],
        "image_phash": image_breakdown["image_phash"],
        "image_color": image_breakdown["image_color"],
        "vienna_overlap": image_breakdown["vienna_overlap"],
        "class_overlap": classes,
        "sign_similarity": round(sign_similarity, 4),
        "overall": overall,
    }
    return {"scores": scores, "tier": tier_for(overall)}


def tier_for(overall: float) -> str:
    if overall >= HIGH_THRESHOLD:
        return "high"
    if overall >= MEDIUM_THRESHOLD:
        return "medium"
    return "low"
