"""Trademark source registry — fan out a search across enabled public sources.

Primary source is TMview (covers ÚPV "CZ" + EUIPO "EM" + WIPO "WO"); ÚPV direct
is an optional enrichment. Results are merged and de-duplicated into one
canonical candidate list for scoring.
"""

from . import base, demo, euipo, tmview, upv
from ..exceptions import ZnamkyError

__all__ = ["base", "demo", "euipo", "tmview", "upv", "territory_offices", "search_all"]


def territory_offices(territories) -> list:
    """Map watched-mark territories (CZ/EU) to TMview office codes."""
    wanted = {t.upper() for t in (territories or [])}
    offices = []
    if "CZ" in wanted:
        offices.append("CZ")
    if "EU" in wanted:
        offices.append("EM")
    if offices:
        offices.append("WO")  # international registrations designating CZ/EU
    return offices or ["CZ", "EM", "WO"]


def search_all(query_text: str, nice_classes=None, territories=None, limit: int = 50) -> dict:
    """Query all enabled sources. Returns {'candidates': [...], 'errors': [...]}.

    A source failure is captured (not raised) so one register being down still
    yields results from the others — mirrors the resilience the watchdog needs.
    """
    offices = territory_offices(territories)
    candidates = []
    errors = []

    if euipo.enabled():
        try:
            candidates.extend(euipo.search(query_text, nice_classes, limit))
        except ZnamkyError as exc:
            errors.append({"source": euipo.SOURCE, "error": str(exc)})

    try:
        candidates.extend(tmview.search(query_text, nice_classes, offices, limit))
    except ZnamkyError as exc:
        errors.append({"source": tmview.SOURCE, "error": str(exc)})

    if upv.enabled() and ("CZ" in offices):
        try:
            candidates.extend(upv.search(query_text, nice_classes, limit))
        except ZnamkyError as exc:
            errors.append({"source": upv.SOURCE, "error": str(exc)})

    if demo.enabled():
        try:
            candidates.extend(demo.search(query_text, nice_classes, limit))
        except ZnamkyError as exc:
            errors.append({"source": demo.SOURCE, "error": str(exc)})

    return {"candidates": base.dedupe(candidates), "errors": errors}
