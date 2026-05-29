# Arbitration — the LLM-judge "second phase"

`run.py --arbitrate` adds a second phase that re-grades **only the rows whose
deterministic assertions failed**. An independent LLM judge decides whether each
failure was a real problem (`INCORRECT` → stays FAIL) or a false positive of the
deterministic check (`CORRECT` → the row flips to PASS). Passing rows never call
the judge, so there is no cost when everything is green.

Why: deterministic checks — especially the `assert_no_tool_call` anti-scrape
guard present in every config — over-fire on legitimate behavior. The recurring
case: the AI uses a plugin tool, gets a URL back, and runs a single `curl`/HEAD to
confirm the URL resolves. That is correct; the check flags it as scraping. LLM
nondeterminism makes such failures intermittent, so retries don't help.

## Running it

```bash
export CDX_EVAL_AUTH_TOKEN=<jwt>                 # daemon, as usual
export OPENAI_BASE_URL=<openai-compatible base>  # e.g. http://localhost:4000/v1
export OPENAI_API_KEY=<key for that endpoint>
export CDX_GRADER_MODEL=<model name>             # optional; default below
tests/promptfoo/run.py --git-url <url> --git-ref <ref> --arbitrate ares-ico.config.yaml
```

Without `--arbitrate`, `run.py` behaves exactly as before (no judge, no `--output`,
identical exit code).

## Judge endpoint (provider-agnostic)

The judge is any **OpenAI-compatible `/chat/completions`** endpoint — the org's
LiteLLM proxy, OpenAI directly, or any gateway. It is called with stdlib `urllib`
(no `requests` dependency), `response_format: {type: json_object}`, `temperature: 0`
— mirroring cdx-daemon `codexis-eval-ops`'s holistic grader.

| Variable | Purpose | Fallback |
|---|---|---|
| `OPENAI_BASE_URL` | base URL incl. `/v1` if the endpoint needs it | `CODEXIS_PUBLIC_LITELLM_BASE_URL` |
| `OPENAI_API_KEY` | bearer token | `CODEXIS_USER_LITELLM_API_KEY` |
| `CDX_GRADER_MODEL` | model name | default `gpt-5.4-mini` |

If neither base+key pair is set under `--arbitrate`, the run aborts with a config
error (exit 2) — it never silently skips the judge.

## How a verdict gates pass/fail

The judge returns `{"verdict": "CORRECT"|"INCORRECT", "confidence": 0.0-1.0,
"rationale": "..."}`.

- `CORRECT` and `confidence ≥ floor` → row flips to **PASS**.
- `CORRECT` and `confidence < floor` → stays **FAIL**, flagged `LOW_CONFIDENCE`.
- `INCORRECT` → stays **FAIL**.
- Malformed reply or HTTP error → stays **FAIL**, flagged `JUDGE_ERROR` (not cached).

A row may fail more than one assertion; the judge sees all of them and is told to
answer `CORRECT` only if **every** flagged failure is a false positive. This keeps a
genuine content failure from being masked by an unrelated false positive.

Floor default is **0.6**, overridable with `--arbitration-confidence`. Tune it after
a few weeks of observed verdicts.

## Per-test criteria ("points")

Each test MAY define what correct vs incorrect tool usage looks like, under
`vars.arbitration`:

```yaml
vars:
  forbidden_regex: '(curl|wget|...)'
  arbitration:
    correct:
      - "A single curl/HEAD against a URL the tool itself returned, to confirm it resolves."
    incorrect:
      - "Using curl/wget to scrape content instead of the dedicated CLI."
      - "Bypassing the CLI to call the upstream site directly."
```

Tests without criteria are still judged (the judge reasons from prompt + response +
tool calls), just less precisely. Seed criteria where the judge frequently has to
guess. Criteria ride in `vars` (the same channel the assertions use), so they reach
the post-processor through promptfoo's `--output` JSON with no extra plumbing.

## Outputs

`run.py --arbitrate` writes (gitignored, archived by CI):

- `tests/promptfoo/.results/arbitration.json` — one record per arbitrated row
  (verdict, confidence, flag, passed, failed assertions, rationale).
- `tests/promptfoo/.results/transcripts/<config>__<idx>.txt` — full audit per
  arbitrated row (both verdicts), with prompt, failed assertions, tool calls, AI
  response, and the judge's decision.

## Cache

Verdicts are cached at `~/.cache/cdx-promptfoo/arbitration-cache.json`, keyed on
`sha256(model | promptfoo_version | prompt | response | assertion_signature)`. The
same failing row re-grades to the same verdict for free on re-runs. Bumping the model
or the promptfoo version (the `# promptfoo:` marker in `requirements.txt`) naturally
invalidates the cache. Wipe it manually with `--arbitration-cache-clear`. `JUDGE_ERROR`
outcomes are never cached, so a transient proxy outage retries next run.

## Tests

`python3 -m pytest tests/promptfoo/test_arbitration.py tests/promptfoo/test_run_arbitration.py`
covers parsing, prompt assembly, verdict mapping, cache, endpoint resolution, and the
`run.py` glue — all with a stubbed judge (no network).
