# katastr - follow-ups

Remaining candidate cleanups. The synthetic-uuid removal, the /simplify quality
pass, the shared-helper reuse (api-client, env_file, createI18n, ErrorMessage),
the dead `?settings` endpoint removal, the shared `recipients_from_body`, the
`stav` classification unification, and the 3-crate -> single-crate consolidation
are all **done**; what's left is recorded below with a rough risk so we can tell
a safe change from a behavior one.

## A. Efficiency - the `check_all` hot path (deliberately deferred)

Deferred by decision: the win is real but bounded, and it must stay polite to
ČÚZK (a state API). All of these live on/near the shared `cdx-watch` check loop.

1. **Poll proceedings concurrently.** `cdx-watch/src/track.rs::check_all` maps
   `poll` over every snapshot sequentially; each `poll` is a blocking ČÚZK GET
   (30s read timeout) + a `store.save`. Wall-clock is the *sum* of round-trips
   instead of the *max*. Fix: a **bounded** thread pool (cap 4-8, not "all at
   once") so ČÚZK isn't hammered. Benefits every watcher plugin.
2. **Reuse one ureq agent.** `api_client.rs::get` builds a fresh `ureq::Agent`
   per call, so each call in a check run does a full TLS handshake to the same
   host. Fix: a process-wide `static AGENT: OnceLock<ureq::Agent>`.
3. **Read the API key once per run** instead of re-reading `.env` on every
   `get()`.

## B. Not-yet-done reuse / dedup

1. **ČÚZK request + auth-error mapping is written twice.**
   `katastr-cli/src/api_client.rs::get` and `katastr-cli/src/settings.rs::validate_key`
   both build a ureq agent, set `ApiKey`/`Accept`, call ČÚZK, and fold the result
   into the same three buckets (401/403 -> `ApiKeyInvalid`, other -> `ApiHttp`,
   transport -> `ApiNetwork`). They have already drifted (read timeout 30s vs
   10s). Extract one private `cuzk_request(key, path) -> Result<ureq::Response>`.
   Risk: unifying forces one timeout + one message shape (a behavior choice).
2. **`KatastrError` notify variants** (`InvalidRecipient` / `Notify` / `Io` /
   `Json`) re-derive their HTTP status in the CGI, which `NotifyError::http_status()`
   already owns (sbirka calls it directly). Low value.

## C. Skipped by /simplify (behavior-changing; kept as candidates)

1. **`postJson` error extraction** (FE `lib/api.ts`) - the two failure branches
   use slightly different guards; a naive merge changes which message is thrown
   in edge cases. Low value. (Now moot: `api.ts` uses the shared client, whose
   `httpError` already centralises this - only relevant if we hand-roll again.)
2. **DO NOT: run `ensure_settings()` only on notify paths.** Recorded as
   rejected - it reintroduces the migration gap we closed. Keep ensure on every
   CGI entry. Only revisit if `ensure()` ever becomes expensive.

## D. FE micro-optimizations (low value - only if the list grows large)

- `deriveTimeline(proc)` (detail/timeline.tsx) and the `ChangeHistory` map+sort
  recompute unmemoized on every render - `useMemo`.
- `ProceedingsList` re-filters and re-renders all rows on selection change;
  `ProceedingRow` is not `React.memo`'d - memoize + stabilise the click handler.

Negligible for a handful of proceedings; premature otherwise.

## E. Cross-plugin - the same `uuid` redundancy still lives elsewhere

The synthetic `uuid` removed from katastr was inherited from the Python cores.
The same redundant per-item `uuid` (+ the `find_by_uuid` O(n) full-directory
scan) still lives in the four Python watcher plugins:

- `znamky` (`znamky_core/tracking.py`)
- `insolvence` (`insolvence_core/tracking.py`)
- `sledovana-judikatura` (in `codexis`)
- `sledovane-dokumenty` (in `codexis`)

Each already has `cislo_rizeni`/a natural key, so `uuid` is a duplicate identity.
`sbirka-listin` and (now) katastr are the clean reference. Fixing each is the
same BE+FE change we did here, per plugin.
