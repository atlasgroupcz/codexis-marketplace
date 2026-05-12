# Plugin E2E Testing — Design

**Date:** 2026-04-21
**Status:** Draft (pending user review)
**Related:** `tests/test-marketplace.py` (install/uninstall acceptance test), `cdx-daemon/codexis-eval-ops/` (promptfoo eval harness — referenced for inspiration, not reused)

## Goal

Verify that each plugin in this marketplace does what it is supposed to do, end-to-end, through a real cdx-daemon chat session. "End-to-end" means: install the plugin into a running daemon, drive a chat that exercises the plugin's skills, and assert on what the model actually did (tool calls, outputs, final text). Failures gate the MR.

Complementary to the existing `tests/test-marketplace.py`, which only checks install/uninstall lifecycle and file presence. This new suite covers *behaviour* — does the plugin actually work from a user's perspective.

## Non-goals

- **Not an LLM benchmark.** We don't care which model the daemon uses; we only care whether the plugin works with whatever the daemon is configured to use. No model is specified anywhere in the tests.
- **Not cross-plugin collision testing.** Each plugin is tested with only itself installed. Collision cases, if they arise, get a separate smoke test later.
- **Not for plugins without `acceptance/e2e/`.** Plugins are added to this suite on an opt-in basis. Missing e2e tests are a warning, not a failure (may revisit).

## Core decisions (from brainstorming)

1. **Grading is per-test-case, not global.** A single test can mix tool-call assertions, output-shape checks, and (rarely) LLM-as-judge.
2. **Multi-turn sequences sharing one chat session.** A "create → list → invoke → delete" lifecycle is one test with five steps, one `chatId`, with values captured between steps.
3. **Runs both in GitLab MR CI and locally.** Same script, same arguments.
4. **CI spins up an ephemeral cdx-daemon per job.** Tests run once the daemon is healthy.
5. **Per-plugin YAML files under `plugins/<name>/acceptance/e2e/`.** Owned by the plugin author, travel with the plugin.
6. **Pytest-based runner** in Python. Consistent with this repo; pytest gives reporting/parallelism/CI integration for free.
7. **Change-detection: `git diff --name-only <base-ref> HEAD`** against the merge-result checkout. Core file changes fall back to running all plugins.
8. **No model configuration.** Daemon's default model handles chat; tests don't know or care.
9. **Install the plugin under test only.** Marketplace is added once; each plugin is installed, tested, and uninstalled in sequence.

## Architecture

### Runtime flow

```
1. MR pipeline creates merge-result branch.
2. CI job spins up cdx-daemon, waits for /actuator/health.
3. CI runs:
     pytest tests/test-plugin-e2e.py
       --daemon    $DAEMON_URL
       --token     $SERVICE_JWT
       --git-url   $CI_REPOSITORY_URL
       --git-ref   $CI_MERGE_REQUEST_REF          # merge-result ref
       --base-ref  origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME
       --changed
4. Suite:
     a. Calls git diff <base-ref>...HEAD locally; derives set of changed plugins.
     b. Calls addMarketplace(GIT, $CI_REPOSITORY_URL, $CI_MERGE_REQUEST_REF) on the daemon.
     c. For each changed plugin <name>:
          - installPlugin(<name>)
          - For each plugins/<name>/acceptance/e2e/*.yaml:
                run the multi-turn chat test against the daemon.
          - uninstallPlugin(<name>)
     d. removeMarketplace
5. pytest exit code gates the MR.
```

### Components

| File | Purpose |
|---|---|
| `tests/_daemon_client.py` | GraphQL client extracted from `test-marketplace.py`. Marketplace add/remove, plugin install/uninstall, entry queries. Shared by both test files. |
| `tests/_chat_runner.py` | `newChat → sendMessage → poll until READY → parse message parts`. Returns structured result `{text, tool_calls: [{name, input, output}]}` per step. Mirrors the pattern in `cdx-daemon/codexis-eval-ops/providers/cdx-graphql.js`, in Python. |
| `tests/_assertions.py` | Interprets the YAML assertion vocabulary against a step result. Raises `AssertionError` with rich messages. |
| `tests/_changed_plugins.py` | `git diff --name-only <base> HEAD` → set of plugin names. Fallback to "all" on core-file changes. |
| `tests/test-plugin-e2e.py` | Pytest collection hook: one parametrized test per YAML file. Per-plugin fixture handles install/uninstall around the plugin's batch of tests. |
| `plugins/<name>/acceptance/e2e/*.yaml` | The tests themselves. Each YAML = one multi-turn chat test. |

### Final repo layout

```
codexis-marketplace/
├── .gitlab-ci.yml                        # new job: plugin-e2e
├── Makefile                              # new targets (test-e2e, test-e2e-changed, test-e2e-only)
├── tests/
│   ├── test-marketplace.py               # unchanged
│   ├── test-plugin-e2e.py                # new
│   ├── _daemon_client.py                 # new (extracted from test-marketplace.py)
│   ├── _chat_runner.py                   # new
│   ├── _assertions.py                    # new
│   └── _changed_plugins.py               # new
└── plugins/<name>/acceptance/
    ├── expected.json                     # existing
    └── e2e/*.yaml                        # new, per-plugin
```

## The YAML test format

A single YAML file = one multi-turn test case = one chat session. Full example:

```yaml
# plugins/codexis/acceptance/e2e/cdxctl-lifecycle.yaml
name: cdxctl-automation-lifecycle
description: create → list → invoke → delete an automation

steps:
  - prompt: "Create an automation named e2e-{{ run_id }} that runs 'echo hi'"
    expect:
      tool_calls:
        - name: cdxctl
          input_contains:
            subcommand: "create"
            name: ~/^e2e-/          # ~ prefix = regex
      capture:
        automationId: "$.tool_calls[0].output.id"

  - prompt: "List automations"
    expect:
      tool_calls:
        - name: cdxctl
          input_contains: { subcommand: "list" }
      output_contains: "{{ automationId }}"

  - prompt: "Invoke the automation {{ automationId }}"
    expect:
      tool_calls:
        - name: cdxctl
          input_contains:
            subcommand: "invoke"
            id: "{{ automationId }}"

  - prompt: "Delete automation {{ automationId }}"
    expect:
      tool_calls:
        - name: cdxctl
          input_contains: { subcommand: "delete" }

  - prompt: "List automations again"
    expect:
      output_not_contains: "{{ automationId }}"
```

### Vocabulary

| Field | Purpose |
|---|---|
| `name` | Required. Used in pytest test ID and failure artifacts. |
| `description` | Optional. Human-readable context. |
| `skip` | Optional. If present (value = reason string), pytest reports skipped. |
| `steps[]` | Required, non-empty. Ordered list of turns in one chat session. |
| `steps[].prompt` | The user message for this turn. Supports `{{ var }}` substitution. |
| `steps[].expect.tool_calls[]` | Ordered list of expected tool invocations in this turn. Each entry: `name` (string, required) + `input_contains` (subset match; values with `~` prefix are regex). Extra tool calls beyond the listed ones are allowed. |
| `steps[].expect.output_contains` | Final assistant text in this turn must contain this literal substring. |
| `steps[].expect.output_matches` | Final assistant text must match this regex. |
| `steps[].expect.output_not_contains` | Negative assertion on final text. |
| `steps[].expect.capture` | Map of `var_name: jsonpath` — extract values from this turn's result into named vars for later steps. |
| `steps[].expect.judge` | `{ rubric: "..." }` — escape hatch for LLM-as-judge. Only for cases where deterministic checks cannot express the correctness criterion. |
| Built-in vars | `{{ run_id }}` = short random hex suffix per test run. Used to avoid collisions with leftover state. |

### Assertion semantics

- **`input_contains`**: recursive subset match. Every key listed in the YAML must be present in the actual tool-call input with a matching value; extra keys in the actual input are fine. A YAML-listed key that's absent in the actual input is a failure. Values prefixed with `~` are Python regex patterns (`re.search`, not `re.fullmatch`).
- **Absent `tool_calls`**: if `tool_calls` is not present in `expect`, no assertion is made on tool invocations in this step (any or none allowed). To assert "no tools were called in this step", use `tool_calls: []`.
- **Ordering of `tool_calls`**: assertions are matched *in order* against the step's tool-call list. First expected entry matches the first actual call whose `name` matches; second expected matches the next one; etc. Intervening calls that don't match any expected entry are allowed.
- **Captures**: run only after all other assertions in the step pass. If a jsonpath resolves to nothing, the test fails with a clear "capture X did not resolve" error. Captured values are used verbatim in later `{{ var }}` substitutions (JSON-encoded for non-strings).
- **`judge` semantics**: the runner sends the step's final text (and optionally the tool-call trace) plus the rubric to the daemon via `newChat(...)` with a judge-prompt template, expecting a structured `{pass: bool, reason: string}` JSON reply. Failure ⇒ the test fails with the judge's `reason` in the assertion message. Used sparingly; deterministic checks are always preferred.
- **No retries.** One run, one verdict. LLM non-determinism causing flakes is a signal to make the prompt more explicit, not to add retries.

## Runner (chat mechanics)

Per-YAML lifecycle:

```python
chat = client.newChat()                       # one chatId per YAML
captured = {"run_id": random_hex(4)}
for step in yaml.steps:
    prompt = substitute(step.prompt, captured)
    exec_id = client.sendMessage(chat.id, prompt)
    result = poll_until_ready(chat.id, exec_id)     # {text, tool_calls}
    run_assertions(step.expect, result, captured)   # raises AssertionError
    captured.update(apply_captures(step.expect.capture, result))
```

**Polling:**
- Interval: `CDX_E2E_POLL_INTERVAL_MS` (default `2000`).
- Timeout: `CDX_E2E_POLL_TIMEOUT_MS` (default `600000`).
- Stop when the last assistant message's `status == READY`.
- If `status == ERROR`, fail immediately with the daemon's error.

**Result parsing** — `parseAssistantMessage(msg) -> {text, tool_calls}`:
- `TextMessagePart` → append `content` to `text`.
- `ToolMessagePart` → append to `tool_calls` with `{name: toolName, input: parsed JSON, output: parsed JSON}`.
- `ThinkingMessagePart` → ignored.

**Isolation:** one `chatId` per YAML (pytest function scope). Install/uninstall is per-plugin, wrapping the plugin's full batch of YAML tests (pytest module-scope fixture keyed on plugin name).

## Change detection

```python
changed_files = git("diff", "--name-only", base_ref, "HEAD").splitlines()
plugins_to_run = set()
for path in changed_files:
    if path.startswith("plugins/"):
        name = path.split("/")[1]
        if (plugins_dir / name).is_dir():
            plugins_to_run.add(name)
    elif path in CORE_TRIGGERS:
        return all_plugins_with_e2e_tests()
return sorted(plugins_to_run)
```

`CORE_TRIGGERS` — explicit list, not a pattern, so added "core" files don't silently expand scope:

```
.claude-plugin/marketplace.json
tests/test-plugin-e2e.py
tests/_daemon_client.py
tests/_chat_runner.py
tests/_assertions.py
tests/_changed_plugins.py
pyproject.toml
```

**CLI flags:**

| Flag | Behaviour |
|---|---|
| *(none)* | Run every plugin that has `acceptance/e2e/`. |
| `--changed` | Use the git-diff logic. Requires `--base-ref` (defaults to `origin/main`). |
| `--only <names>` | Comma-separated plugin names; overrides everything else. Fast local iteration. |

**Edge cases:**
- Plugin directory deleted in the MR → not included (the `is_dir()` guard).
- Plugin added with no `acceptance/e2e/` directory → skipped with a pytest warning, not a failure.
- Docs-only changes inside a plugin → still trigger that plugin's e2e run. Simpler; docs-only changes are rare in isolation.

## CI integration

New GitLab CI job, parallel to the existing marketplace acceptance job:

```yaml
plugin-e2e:
  stage: test
  needs: [daemon-up]                         # same dependency as test-marketplace
  script:
    - pytest tests/test-plugin-e2e.py
        --daemon    "$DAEMON_URL"
        --token     "$SERVICE_JWT"
        --git-url   "$CI_REPOSITORY_URL"
        --git-ref   "$CI_MERGE_REQUEST_REF"
        --base-ref  "origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME"
        --changed
        --junitxml=test-results/plugin-e2e-junit.xml
        -v
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
  artifacts:
    when: always
    paths:
      - test-results/plugin-e2e-junit.xml
      - test-results/transcripts/
    reports:
      junit: test-results/plugin-e2e-junit.xml
```

**Secrets / env needed:** `DAEMON_URL`, `SERVICE_JWT`. No LLM credentials (daemon handles that).

**Failure artifacts** — debuggability is the single biggest deciding factor for whether engineers actually use this suite. The runner writes `test-results/transcripts/<plugin>/<yaml-name>.md` for every failed test:

```markdown
# cdxctl-automation-lifecycle

## Step 1 — PASS
User: Create an automation named e2e-ab12 that runs 'echo hi'
Tool call: cdxctl({"subcommand":"create","name":"e2e-ab12","cmd":"echo hi"})
  → {"id":"auto_7f3","name":"e2e-ab12",...}
Assistant: Created automation auto_7f3...

## Step 2 — FAIL
User: List automations
Tool call: cdxctl({"subcommand":"list"})
  → {"automations":[]}
Assistant: No automations found.

Assertion failed: output_contains "auto_7f3" — actual text did not contain it.
```

Uploaded as a GitLab artifact so reviewers see the exact chat without re-running.

## Local development workflow

**Core constraint:** cdx-daemon installs plugins via `addMarketplace(GIT, url, ref)` — it clones from a remote. Uncommitted / unpushed changes are invisible to it. Every iteration requires a push. This is inherent to the daemon's architecture and matches how `test-marketplace.py` already operates.

**Workflow:**

```
1. Edit plugins/codexis/... and/or plugins/codexis/acceptance/e2e/*.yaml
2. git commit && git push origin my-feature-branch
3. make test-e2e-only PLUGIN=codexis
4. Read the failure transcript, fix, repeat from step 2.
```

**Makefile targets:**

```makefile
test-e2e:
	pytest tests/test-plugin-e2e.py \
	  --daemon   $${DAEMON_URL:?set DAEMON_URL} \
	  --token    $${SERVICE_JWT:?set SERVICE_JWT} \
	  --git-url  $${TEST_GIT_URL:?set TEST_GIT_URL} \
	  --git-ref  $${TEST_GIT_REF:?set TEST_GIT_REF} \
	  --base-ref $${TEST_BASE_REF:-origin/main} \
	  -v

test-e2e-changed: ; $(MAKE) test-e2e PYTEST_EXTRA="--changed"
test-e2e-only:    ; $(MAKE) test-e2e PYTEST_EXTRA="--only $(PLUGIN)"
```

Same binary in CI and locally. The only difference is *who fills in `TEST_GIT_REF`* — CI uses `$CI_MERGE_REQUEST_REF`, local uses whatever branch you just pushed.

## Open questions / YAGNI placeholders

These are deliberately **not** in scope for v1; if a real need appears, revisit.

- **Cross-plugin collision smoke test** — separate suite with all plugins installed. Deferred until a collision actually happens.
- **External-setup hooks** (pre-test REST calls, file uploads) — could be added as a top-level `setup:`/`teardown:` in YAML. Deferred until a plugin genuinely needs it.
- **Strict mode for missing `acceptance/e2e/`** — fail rather than warn for new plugins. Deferred; enable once the suite covers the existing plugins.
- **Parallelism across plugins** — pytest-xdist etc. Deferred; first establish the suite works, then optimize.
