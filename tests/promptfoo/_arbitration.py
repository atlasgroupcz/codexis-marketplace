"""Arbitration: an LLM-judge "second phase" over deterministically-failing e2e rows.

The deterministic promptfoo assertions (tool-call shape, regex, GraphQL/file state,
the `assert_no_tool_call` anti-scrape check) over-fire on legitimate-but-flagged AI
behavior — e.g. a single `curl`/HEAD to confirm a URL the plugin returned actually
resolves. LLM nondeterminism makes those failures intermittent.

When `run.py --arbitrate` is set, every row whose deterministic assertions FAILED is
re-graded by an independent LLM judge against author-defined "correctness criteria"
(`vars.arbitration.{correct,incorrect}`). The judge decides CORRECT (the deterministic
check was a false positive → flip the row to PASS) or INCORRECT (a real failure → stay
FAIL). Passing rows never call the judge — no cost.

Judge transport is provider-agnostic: any OpenAI-compatible `/chat/completions` endpoint
(the org LiteLLM proxy, OpenAI, a gateway). Mirrors cdx-daemon `codexis-eval-ops`'s
holistic-grader (`OPENAI_BASE_URL` / `OPENAI_API_KEY` / `CDX_GRADER_MODEL`,
`response_format: json_object`, `temperature: 0`), implemented with stdlib urllib to
match this repo's HTTP convention (`_daemon_client.py`).
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Mapping, Optional

# Reuse the single source of truth for how the provider smuggles tool calls
# through promptfoo's output string (sentinel-bracketed JSON suffix).
from assertions import TOOL_CALLS_SENTINEL, _extract_tool_calls

DEFAULT_CONFIDENCE = 0.6
DEFAULT_MODEL = "gpt-5.4-mini"  # matches cdx-daemon codexis-eval-ops default
DEFAULT_TIMEOUT_S = 60.0

SYSTEM_PROMPT = (
    "You are a strict but context-aware evaluator of an AI agent's TOOL USAGE in an "
    "automated end-to-end test for a Czech/EU legal-AI plugin marketplace. A deterministic "
    "check flagged the AI's behavior as a potential failure, but deterministic checks cannot "
    "tell legitimate tool use from misuse (e.g. a single curl/HEAD to verify a URL the plugin "
    "returned is fine; curl to scrape content or to bypass the dedicated CLI tool is not).\n\n"
    "Using the author-defined correctness criteria when provided, plus the prompt, response and "
    "tool-call transcript, decide whether the AI's ACTUAL behavior was CORRECT (the deterministic "
    "check was a false positive) or INCORRECT (a real failure). Be conservative: if the behavior "
    "plausibly matches an 'incorrect' criterion, or you are unsure, choose INCORRECT and/or lower "
    "your confidence.\n\n"
    "A row may have failed MORE THAN ONE deterministic check (all failures are listed). "
    "Answer CORRECT only if EVERY listed failure is a false positive and the AI fully satisfied "
    "the test's intent. If any failure reflects a genuine problem (wrong/missing required content, "
    "actual scraping, bypassing the dedicated tool), answer INCORRECT.\n\n"
    "Reply ONLY with a compact JSON object, no markdown, of the form:\n"
    '{"verdict":"CORRECT"|"INCORRECT","confidence":0.0,"rationale":"<=300 chars"}'
)


class ArbitrationConfigError(RuntimeError):
    """Raised when arbitration is requested but no judge endpoint/key is configured."""


@dataclass
class FailedCase:
    config: str
    test_idx: int
    description: str
    prompt: str
    response_text: str
    tool_calls: list
    failed_assertions: list  # [{"type", "value", "reason"}]
    criteria: Optional[dict]  # {"correct": [...], "incorrect": [...]} or None


@dataclass
class Verdict:
    verdict: str  # "CORRECT" | "INCORRECT"
    confidence: float
    rationale: str


@dataclass
class ArbitrationResult:
    passed: bool       # should the failing row flip to PASS?
    verdict: str
    confidence: float
    rationale: str
    flag: str = ""     # "" | "LOW_CONFIDENCE" | "JUDGE_ERROR"


# --------------------------------------------------------------------------- #
# Parse promptfoo --output JSON into failing cases.
# --------------------------------------------------------------------------- #

def split_output(output: str) -> tuple[str, list]:
    """Separate the AI's text from the sentinel-wrapped tool_calls suffix."""
    if not isinstance(output, str):
        return "", []
    text = output.split(TOOL_CALLS_SENTINEL, 1)[0].rstrip()
    return text, _extract_tool_calls(output)


def parse_failures(result_json: dict, config: str) -> list[FailedCase]:
    """Extract one FailedCase per failing row from a promptfoo --output JSON.

    Handles both the nested `{"results": {"results": [...]}}` shape (promptfoo
    0.121.x) and a bare top-level `{"results": [...]}` for resilience.
    """
    res = result_json.get("results")
    rows = res.get("results") if isinstance(res, dict) else res
    cases: list[FailedCase] = []
    for row in rows or []:
        if row.get("success"):
            continue
        vars_ = row.get("vars") or {}
        prompt = vars_.get("prompt") or ""
        description = (row.get("testCase") or {}).get("description") or prompt
        text, tool_calls = split_output((row.get("response") or {}).get("output") or "")
        failed: list[dict] = []
        for c in ((row.get("gradingResult") or {}).get("componentResults")) or []:
            if c.get("pass"):
                continue
            a = c.get("assertion") or {}
            failed.append({"type": a.get("type"), "value": a.get("value"),
                           "reason": c.get("reason") or ""})
        cases.append(FailedCase(
            config=config,
            test_idx=row.get("testIdx", 0),
            description=description,
            prompt=prompt,
            response_text=text,
            tool_calls=tool_calls,
            failed_assertions=failed,
            criteria=vars_.get("arbitration"),
        ))
    return cases


# --------------------------------------------------------------------------- #
# Judge prompt assembly.
# --------------------------------------------------------------------------- #

def build_judge_messages(case: FailedCase) -> list[dict]:
    parts = [
        f"TEST INTENT:\n  {case.description or case.prompt}",
        f"USER PROMPT TO AI:\n  {case.prompt}",
        f"AI'S RESPONSE (text):\n  {case.response_text}",
    ]
    if case.tool_calls:
        tc = "\n".join(
            f"  {i + 1}. {t.get('name')}: "
            f"{json.dumps(t.get('input') or {}, ensure_ascii=False)}"
            for i, t in enumerate(case.tool_calls))
    else:
        tc = "  (none)"
    parts.append("AI'S TOOL-CALL SEQUENCE:\n" + tc)

    fa = "\n".join(
        f"  - type={a.get('type')} value={a.get('value')}\n    reason: {a.get('reason')}"
        for a in case.failed_assertions) or "  (none)"
    parts.append("DETERMINISTIC ASSERTION(S) THAT FAILED:\n" + fa)

    if case.criteria:
        crit: list[str] = []
        for c in case.criteria.get("correct") or []:
            crit.append(f"  CORRECT (acceptable): {c}")
        for c in case.criteria.get("incorrect") or []:
            crit.append(f"  INCORRECT (not acceptable): {c}")
        if crit:
            parts.append("AUTHOR-DEFINED CORRECTNESS CRITERIA:\n" + "\n".join(crit))

    parts.append(
        "DECIDE whether the AI's actual tool usage was CORRECT or INCORRECT. "
        'Reply ONLY with JSON: '
        '{"verdict":"CORRECT"|"INCORRECT","confidence":0.0,"rationale":"<=300 chars"}')

    return [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user", "content": "\n\n".join(parts)},
    ]


# --------------------------------------------------------------------------- #
# Verdict parsing.
# --------------------------------------------------------------------------- #

def parse_verdict(raw: str) -> Verdict:
    """Parse the judge's JSON reply, tolerating ```json fences. Raises ValueError."""
    if not isinstance(raw, str):
        raise ValueError(f"judge reply not a string: {raw!r}")
    s = raw.strip()
    if s.startswith("```"):
        s = s.strip("`")
        s = re.sub(r"^\s*json\s*", "", s, flags=re.IGNORECASE).strip()
    try:
        data = json.loads(s)
    except json.JSONDecodeError as e:
        raise ValueError(f"judge reply was not JSON: {raw!r}") from e
    if not isinstance(data, dict) or "verdict" not in data:
        raise ValueError(f"judge reply missing 'verdict': {raw!r}")
    try:
        confidence = float(data.get("confidence", 0.0))
    except (TypeError, ValueError):
        confidence = 0.0
    return Verdict(verdict=str(data["verdict"]).strip().upper(),
                   confidence=confidence,
                   rationale=str(data.get("rationale", "")).strip())


# --------------------------------------------------------------------------- #
# Cache key.
# --------------------------------------------------------------------------- #

def assertion_signature(failed_assertions: list) -> str:
    """Stable signature of the failed assertions (type+value only; reason excluded)."""
    sig = [{"type": a.get("type"), "value": a.get("value")}
           for a in (failed_assertions or [])]
    return json.dumps(sig, sort_keys=True, ensure_ascii=False)


def cache_key(model: str, pf_version: str, prompt: str, response: str,
              assertion_sig: str) -> str:
    h = hashlib.sha256()
    h.update(f"{model}|{pf_version}|{prompt}|{response}|{assertion_sig}".encode("utf-8"))
    return h.hexdigest()


# --------------------------------------------------------------------------- #
# Arbitration: verdict -> pass/fail, with cache + error handling.
# --------------------------------------------------------------------------- #

def arbitrate(case: FailedCase, *,
              call: Callable[[list], str],
              model: str,
              pf_version: str,
              floor: float = DEFAULT_CONFIDENCE,
              cache: Optional[dict] = None) -> ArbitrationResult:
    """Re-grade one failing case. `call(messages) -> raw_json_str` is the judge."""
    key = cache_key(model, pf_version, case.prompt, case.response_text,
                    assertion_signature(case.failed_assertions))
    if cache is not None and key in cache:
        return ArbitrationResult(**cache[key])

    try:
        verdict = parse_verdict(call(build_judge_messages(case)))
    except Exception as e:  # HTTP failure or malformed reply → keep FAIL, flag it
        return ArbitrationResult(passed=False, verdict="ERROR", confidence=0.0,
                                 rationale=str(e)[:300], flag="JUDGE_ERROR")

    if verdict.verdict == "CORRECT" and verdict.confidence >= floor:
        result = ArbitrationResult(True, verdict.verdict, verdict.confidence,
                                   verdict.rationale, "")
    elif verdict.verdict == "CORRECT":
        result = ArbitrationResult(False, verdict.verdict, verdict.confidence,
                                   verdict.rationale, "LOW_CONFIDENCE")
    else:
        result = ArbitrationResult(False, verdict.verdict, verdict.confidence,
                                   verdict.rationale, "")

    if cache is not None:  # only real verdicts are cached, never JUDGE_ERROR
        cache[key] = {"passed": result.passed, "verdict": result.verdict,
                      "confidence": result.confidence, "rationale": result.rationale,
                      "flag": result.flag}
    return result


# --------------------------------------------------------------------------- #
# Judge transport (OpenAI-compatible) + endpoint resolution + file cache.
# Not unit-tested (network) — kept thin so `arbitrate`'s `call` can be stubbed.
# --------------------------------------------------------------------------- #

def resolve_endpoint(env: Optional[Mapping[str, str]] = None) -> tuple[str, str, str]:
    """Return (base_url, api_key, model) for the judge, or raise ArbitrationConfigError.

    Prefers the cdx-daemon eval-ops names (OPENAI_BASE_URL / OPENAI_API_KEY /
    CDX_GRADER_MODEL); falls back to the org's host-injected LiteLLM vars.
    """
    env = os.environ if env is None else env
    base = env.get("OPENAI_BASE_URL") or env.get("CODEXIS_PUBLIC_LITELLM_BASE_URL")
    key = env.get("OPENAI_API_KEY") or env.get("CODEXIS_USER_LITELLM_API_KEY")
    model = env.get("CDX_GRADER_MODEL") or DEFAULT_MODEL
    if not base or not key:
        raise ArbitrationConfigError(
            "arbitration requires an OpenAI-compatible judge endpoint: set "
            "OPENAI_BASE_URL + OPENAI_API_KEY (or CODEXIS_PUBLIC_LITELLM_BASE_URL + "
            "CODEXIS_USER_LITELLM_API_KEY)")
    return base, key, model


def make_judge_caller(base_url: str, api_key: str, model: str,
                      timeout: float = DEFAULT_TIMEOUT_S) -> Callable[[list], str]:
    """Build a `call(messages) -> content_str` against an OpenAI-compatible endpoint."""
    url = base_url.rstrip("/") + "/chat/completions"

    def call(messages: list) -> str:
        body = json.dumps({
            "model": model,
            "messages": messages,
            "response_format": {"type": "json_object"},
            "temperature": 0,
        }).encode("utf-8")
        req = urllib.request.Request(url, data=body, method="POST", headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}",
        })
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                payload = json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as e:
            detail = e.read().decode("utf-8", "replace")[:500]
            raise RuntimeError(f"judge HTTP {e.code}: {detail}") from e
        try:
            return payload["choices"][0]["message"]["content"]
        except (KeyError, IndexError, TypeError) as e:
            raise RuntimeError(f"judge reply missing content: {str(payload)[:300]}") from e

    return call


def arbitrate_failures(result_json: dict, config: str, *,
                       call: Callable[[list], str],
                       model: str,
                       pf_version: str,
                       floor: float = DEFAULT_CONFIDENCE,
                       cache: Optional[dict] = None) -> list[tuple[FailedCase, ArbitrationResult]]:
    """Re-grade every failing row in one config's promptfoo --output JSON."""
    return [
        (case, arbitrate(case, call=call, model=model, pf_version=pf_version,
                         floor=floor, cache=cache))
        for case in parse_failures(result_json, config)
    ]


def recompute_pass(original_ok: bool,
                   outcomes: list[tuple[FailedCase, ArbitrationResult]]) -> bool:
    """Config-level pass after arbitration.

    A config that promptfoo already passed stays passed. A failed config passes
    only if it had failing rows AND the judge flipped every one of them to PASS.
    A failed config with nothing to arbitrate (e.g. an unparseable provider crash)
    stays failed.
    """
    if original_ok:
        return True
    if not outcomes:
        return False
    return all(result.passed for _, result in outcomes)


def transcript_slug(config: str, test_idx: int) -> str:
    """Filesystem-safe `<config>__<idx>` slug for a per-failure transcript file."""
    return re.sub(r"[^a-zA-Z0-9._-]", "_", f"{config}__{test_idx}")[:200]


def format_transcript(case: FailedCase, result: ArbitrationResult,
                      final_pass: bool) -> str:
    """Human-readable audit record for one arbitrated failure."""
    lines = [
        f"config:       {case.config}",
        f"test_idx:     {case.test_idx}",
        f"description:  {case.description}",
        f"verdict:      {result.verdict}  (confidence={result.confidence})",
        f"flag:         {result.flag or '-'}",
        f"final result: {'PASS (overruled by judge)' if final_pass else 'FAIL'}",
        f"rationale:    {result.rationale}",
        "",
        "PROMPT:",
        f"  {case.prompt}",
        "",
        "FAILED DETERMINISTIC ASSERTIONS:",
    ]
    for a in case.failed_assertions:
        lines.append(f"  - {a.get('type')} {a.get('value')}")
        lines.append(f"    reason: {a.get('reason')}")
    lines += ["", "TOOL CALLS:"]
    for i, t in enumerate(case.tool_calls):
        lines.append(f"  {i + 1}. {t.get('name')}: "
                     f"{json.dumps(t.get('input') or {}, ensure_ascii=False)}")
    lines += ["", "AI RESPONSE (text):", case.response_text, ""]
    return "\n".join(lines)


def default_cache_path() -> Path:
    return Path(os.path.expanduser("~/.cache/cdx-promptfoo/arbitration-cache.json"))


def load_cache(path: Optional[Path] = None) -> dict:
    path = path or default_cache_path()
    try:
        return json.loads(path.read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        return {}


def save_cache(cache: dict, path: Optional[Path] = None) -> None:
    path = path or default_cache_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(cache, ensure_ascii=False))
