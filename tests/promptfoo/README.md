# Promptfoo POC — cdx-at port

Migrates `plugins/cdx-at/acceptance/e2e/cdx-at-search.yaml` to
[promptfoo](https://promptfoo.dev) to validate the migration path.

## Why

Two specific pains in the legacy runner that promptfoo addresses:

- **Brittle regex-on-response checks.** A pattern like
  `(?s)990.{0,400}zastav` or `výslovného ustanovení` is a poor proxy
  for "did the AI answer correctly." `llm-rubric` lets us write the
  intent directly and hand grading off to a small judge model.
- **No semantic regression detection.** The legacy runner is
  pass/fail per row; promptfoo gives diffable runs, web UI, and
  per-assertion scoring out of the box.

What we keep from the legacy runner:

- `DaemonClient` + `ChatRunner` (the schema-realigned chat protocol,
  JWT auto-refresh, FileEntry adapter). They live one directory up
  and are imported wholesale by `provider.py`.
- The structural "did the model call cdx-at search?" axis, expressed
  via a custom Python assertion in `assertions.py` instead of the
  legacy YAML's `tool_call.input_matches`.

## Files

| File | Role |
|---|---|
| `provider.py` | Custom promptfoo provider. One chat per test row. Returns AI text as `output`, structured `tool_calls` as `metadata`. |
| `assertions.py` | `assert_tool_call` + `assert_tool_count_max`. Walk the assertion `context` for the provider metadata across promptfoo versions. |
| `cdx-at.config.yaml` | promptfoo eval config — port of the legacy cdx-at YAML. |

## Prereqs

- promptfoo CLI (npx works fine):
  ```bash
  npx --yes promptfoo --version
  ```
- `ANTHROPIC_API_KEY` exported (the `llm-rubric` assertions use Anthropic).
- A running cdx-daemon at `$CDX_EVAL_DAEMON` (default
  `http://localhost:8086`) with the `cdx-at` plugin installed
  (the legacy `test-marketplace.py` is the easiest way to set that up
  one-off).

## Run

```bash
# 1. Get a Keycloak access token.
#    For local dev: log in once at http://localhost:8088, then mint it via
#    oauth2-proxy with your browser session cookie:
COOKIE='_oauth2_proxy=…'   # from browser DevTools → Application → Cookies
export CDX_EVAL_AUTH_TOKEN=$(curl -sS -D - -o /dev/null \
  -H "Cookie: $COOKIE" http://localhost:4182/oauth2/auth |
  awk -F': ' '/^X-Auth-Request-Access-Token:/ {print $2}' | tr -d '\r')
#
#    For CI: use a long-lived service-account JWT.

# 2. (Optional) Override the daemon GraphQL URL — default is localhost.
export CDX_EVAL_GRAPHQL_URL=http://localhost:8086/graphql

# 3. Run.
npx promptfoo eval -c tests/promptfoo/cdx-at.config.yaml

# 4. View the results.
npx promptfoo view
```

If a run outlasts the token's TTL (~30 min in local Keycloak), re-export
`CDX_EVAL_AUTH_TOKEN` and re-run. Eval batches are short by design;
long-running suites belong in the legacy `test-plugin-e2e.py` runner
which has its own auto-refresh path.

## Migration scope (if we go full)

This POC covers ~2 prompts of one plugin. A full migration would mean:

- One config per plugin, mirroring `plugins/<name>/acceptance/e2e/`
  layout. (Or a single combined config with tags.)
- Multi-step / `capture:` flows (e.g. cdxctl-marketplace-git's 5
  steps where one captures a marketplace ID for the next) need the
  conversation pattern in promptfoo — workable, but a per-yaml port.
- State-via-GraphQL assertions (used by cdxctl tests) become a third
  Python assertion: `assert_state_graphql` calling our
  `DaemonClient.gql_data()`.
- The `setup:` block (file uploads — used by ocr fixtures and the
  katastr API key) maps to a `beforeEach` extension hook in JS, or
  simply runs once before `promptfoo eval` from a wrapper script.

Estimated full-port effort: ~1-2 days, dominated by the cdxctl
multi-step tests' `capture:` translations.
