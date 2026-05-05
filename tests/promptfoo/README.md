# Promptfoo POC — cdx-at port

Migrates `plugins/cdx-at/acceptance/e2e/cdx-at-search.yaml` from the
custom `tests/test-plugin-e2e.py` runner to [promptfoo](https://promptfoo.dev).

## Why promptfoo

Not for AI grading — the deterministic assertions we already have
(regex on response, structural tool_call match, state-via-GraphQL for
side effects) are good enough. The motivation is to **stop maintaining
a custom test framework** and use a standard, maintained tool that
gives us:

- standard YAML test format
- web UI / diffable runs
- multi-provider support (could test the same prompt against different
  daemon configs)
- per-assertion scoring

What we keep:

- `DaemonClient` + `ChatRunner` (the schema-realigned chat protocol).
  They live one directory up and are imported wholesale by `provider.py`
  so chat-protocol changes stay in one place.

## Files

| File | Role |
|---|---|
| `provider.py` | Custom promptfoo provider. One chat per test row. Returns AI text as `output`, structured `tool_calls` as `metadata`. |
| `assertions.py` | Three deterministic Python assertions: `assert_tool_call`, `assert_tool_count_max`, `assert_state_graphql`. |
| `cdx-at.config.yaml` | promptfoo eval config — port of the legacy cdx-at YAML. |

## Assertions available

| Assertion | When to use | Maps to legacy YAML |
|---|---|---|
| built-in `regex` / `contains` / `not-contains` | Pattern in the AI's response text | `response.matches` / `response.contains` |
| `assert_tool_call` | Did the AI invoke a specific tool with matching args? | `tool_call.{name,input_matches}` |
| `assert_tool_count_max` | "Did AI use the right tool?" — see note below | `tool_calls_max` |
| `assert_state_graphql` | Daemon-side state assertion (e.g. "automation X exists") | `state.graphql` |

**On `tool_calls_max`**: it's not a budget knob; it's a sanity signal.
A healthy single-search workflow is ~1-2 work calls. If the count balloons
past 8 on a single-step search, the AI almost certainly fell back to
`curl`/scraping because our binary errored (PATH issue, auth 401, missing
env var, etc.). Set the cap to a realistic upper bound for the workflow,
then trust it — when it fires, investigate the daemon/plugin, don't
loosen the cap. Prompts should sound like real users; do **not** add
"only do 1 search please" framing to artificially shrink the count.

`state.graphql` supports the same operators the legacy framework had:
`count`, `contains`, `not_contains`, `matches`, `equals`, plus optional
`jsonpath` to drill into the response.

## Prereqs

- promptfoo CLI (npx works fine):
  ```bash
  npx --yes promptfoo --version
  ```
- A running cdx-daemon at `$CDX_EVAL_GRAPHQL_URL` with the `cdx-at`
  plugin installed (the legacy `test-marketplace.py` is the easiest
  way to set that up one-off).

## Run

```bash
# 1. Get a Keycloak access token. For local dev, mint one via oauth2-proxy
#    after logging in once at http://localhost:8088:
COOKIE='_oauth2_proxy=…'   # from browser DevTools → Application → Cookies
export CDX_EVAL_AUTH_TOKEN=$(curl -sS -D - -o /dev/null \
  -H "Cookie: $COOKIE" http://localhost:4182/oauth2/auth |
  awk -F': ' '/^X-Auth-Request-Access-Token:/ {print $2}' | tr -d '\r')
#
# For CI: use a long-lived service-account JWT.

# 2. (Optional) Override the daemon GraphQL URL — default is localhost.
export CDX_EVAL_GRAPHQL_URL=http://localhost:8086/graphql

# 3. Run.
npx promptfoo eval -c tests/promptfoo/cdx-at.config.yaml

# 4. View the results.
npx promptfoo view
```

If a run outlasts the token's TTL (~30 min in local Keycloak), re-export
`CDX_EVAL_AUTH_TOKEN` and re-run.

## Migration scope (full port)

- One config per plugin, mirroring `plugins/<name>/acceptance/e2e/`
  layout. (Or a single combined config with tags.)
- Multi-step / `capture:` flows (e.g. cdxctl-marketplace-git's 5
  steps where one captures a marketplace ID for the next) need the
  conversation pattern in promptfoo — workable, but a per-yaml port.
- The `setup:` block (file uploads — used by ocr fixtures and the
  katastr API key) maps to a `beforeEach` extension hook in JS, or
  runs once before `promptfoo eval` from a wrapper script.

Estimated full-port effort: ~1-2 days, dominated by the cdxctl
multi-step tests' `capture:` translations.
