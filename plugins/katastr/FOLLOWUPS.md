# katastr - follow-ups

Candidate cleanups found while removing the synthetic `uuid` and running
`/simplify` over the whole plugin (BE + FE). None of these are blocking; they are
recorded so the next pass can pick them up. Each notes the rough risk so we can
tell a safe swap from a behavior change.

Verified state at time of writing: `cargo test` 22 green + clippy clean; FE
`tsc` + 30 vitest green + eslint clean.

## A. Skipped by /simplify (real candidates, but each changes behavior)

These were left out of the cleanup because they are not behavior-preserving, not
because they are wrong. Several probably *are* better than what we ship now -
they just need a deliberate decision + a test update, not a silent simplify.

1. **Unify the `stav` classification** (FE)
   - `components/stav-badge.tsx` (`getStavColor` + `shortStav`) and
     `lib/timeline.ts` (`NEGATIVE_TERMINAL_STEMS` / `isNegativeTerminal`) each
     keyword-match the same ČÚZK `stav` string with *disjoint* vocabularies.
   - They already disagree: a rejected proceeding (`Zamítnuto`) renders a red
     terminal node in the timeline but a neutral outline chip in the badge.
   - Fix: one `lib/stav.ts` mapping `stav -> category` (in-progress /
     positive-terminal / negative-terminal / plomba / neutral), consumed by both.
   - Risk: **changes visible rendering** (rejected badge would go red). Needs a
     product decision + snapshot/test update. Arguably a latent bug fix.

2. **Deduplicate the ČÚZK request + auth-error mapping** (BE)
   - `katastr-core/src/api_client.rs::get` and
     `katastr-core/src/settings.rs::validate_key` build a ureq agent, set the
     `ApiKey`/`Accept` headers, and fold the result into the same three buckets
     (401/403 -> `ApiKeyInvalid`, other status -> `ApiHttp`, transport ->
     `ApiNetwork`). `validate_key` is basically `get(HEALTH_PATH)` with an
     explicit key and the body discarded.
   - Fix: one private `cuzk_request(key, path) -> Result<ureq::Response>` that
     owns agent + headers + error mapping; `get` and `validate_key` layer their
     specifics on top.
   - Risk: the two copies have **drifted** - read timeout is 30s in `get` vs 10s
     in `validate_key`, and only `get`'s `ApiHttp` carries the path/body snippet.
     Unifying forces one timeout + one message shape (a behavior change to pick).

3. **`postJson` error extraction** (FE `lib/api.ts`)
   - The `!response.ok` branch and the `ok === false` branch both open-code the
     `typeof json === 'object' && 'error' in json` guard, with *different*
     strictness (`String(error)` vs `typeof === 'string' && err`).
   - Fix: one `pickError(json): string | undefined` helper.
   - Risk: the two guards differ in edge cases (empty/non-string `error`), so a
     naive merge changes which message is thrown. Low value.

4. **DO NOT: run `ensure_settings()` only on notify paths** (BE)
   - The efficiency reviewer suggested calling `ensure_settings()` only from
     `handle_notify_view` / `post_save_notify_settings` instead of every CGI
     request. **Recorded as rejected**: it reintroduces the exact migration gap we
     just closed - an app-only user who never opens notify settings would never
     get their `settings.json` migrated. Keep ensure on every entry point. Only
     revisit if `ensure()` ever becomes expensive (today it is one JSON read).

## B. Efficiency - the `check_all` hot path (BE, biggest real win)

All three compound on the same loop; #1 dominates real-world latency once a user
tracks more than a couple of proceedings. #1 and the agent live in the shared
`cdx-watch` crate, so a fix benefits every watcher plugin.

1. **Poll proceedings concurrently.** `cdx-watch/src/track.rs::check_all` maps
   `poll` over every snapshot sequentially; each `poll` is a blocking ČÚZK GET
   (30s read timeout) + a `store.save`. Wall-clock is the *sum* of round-trips
   instead of the *max*. The polls are independent (distinct URLs + distinct
   slugs, no write contention). Fix: bounded thread pool (`std::thread::scope`,
   cap 4-8) or rayon. ~20 proceedings at ~300ms: ~6s -> ~0.5s.
2. **Reuse one ureq agent.** `api_client.rs::get` builds a fresh
   `ureq::Agent` per call, so each of N calls does a full TLS handshake to the
   same host. Fix: process-wide `static AGENT: OnceLock<ureq::Agent>` (keep-alive
   pool). Saves N-1 handshakes on the check-all path.
3. **Read the API key once per run.** `get()` calls `read_api_key()` which
   re-reads + re-parses `.env` on every call. Memoize per run.
4. **`notify::dispatch`** runs `send_email` then `send_inapp` serially; the
   in-app send is independent and could overlap. Low priority (rare path).

## C. Reuse - swap local copies for shared helpers

Each is a "swap for the shared thing" with a small caveat; do as one consolidation
pass.

1. **FE `lib/api.ts`** reimplements `@workspace/ui/lib/api-client`
   (`createApiClient`) almost line-for-line (~55 lines). The shared `getJson`
   even surfaces the server `error` on non-200 (we currently hide it). Caveat:
   the shared `postJson` fallback message is English (`"Request failed"`) vs our
   Czech `"Akce selhala: <action>"`.
2. **BE `settings.rs`** hand-rolls `.env` read/write that
   `cdx_common::env_file` (`read_env_var` / `write_env_file`) already provides -
   the module was written for exactly this case and is currently dead. Enable the
   `env-file` feature (`fs` is already on).
3. **FE `lib/i18n.ts`** duplicates `@workspace/ui/lib/i18n` (`createI18n`).
   Caveat: adopting it drops the dev `debug` flag.
4. **FE `components/error-message.tsx`** is a subset copy of
   `@workspace/ui/components/error-message`. Caveat: shared button is text-only
   (loses the small `RefreshCw` icon).

## D. Altitude / structure

1. **Consolidate katastr to a single crate.** katastr is the *only* Rust plugin
   using a 3-crate Cargo workspace (`katastr-core` + `katastr-cgi` +
   `katastr-cli`). Every other Rust plugin (ares, zrsr, cdx-sk, leciva,
   register-zmluv, nrpzs, sbirka-listin) is a single crate with `src/lib.rs` +
   `src/main.rs` + `src/cgi_main.rs` and multiple `[[bin]]` targets. Collapsing
   katastr to match = one Cargo.toml, one target dir, faster builds, less
   boilerplate. Structural refactor: move sources, merge Cargo.toml, adjust
   `src/build.sh`, fix imports.
2. **Drop the dead `?settings` GET endpoint.** `katastr-cgi::handle_settings` +
   `SettingsResponse` return a second key-status shape that the SPA never
   requests (the FE reads `api_key_configured` / `api_key_masked` off the
   overview response). Verify no e2e/parity consumer first, then remove.
3. **Shared `recipients_from_body` helper.** Both `katastr-cgi` and
   `sbirka-listin` CGIs hand-roll the same "pull `recipients` array, filter to
   strings, `without_self`, `None` means leave untouched" block. Belongs in
   `cdx-notify`.
4. **`KatastrError` notify variants** (`InvalidRecipient` / `Notify` / `Io` /
   `Json`) re-declare `NotifyError`'s and the CGI re-derives their HTTP status,
   which `NotifyError::http_status()` already owns (sbirka calls it directly).
   Low value; only bites if the shared status policy changes.

## E. FE micro-optimizations (low value - only if the list grows large)

- `deriveTimeline(proc)` (detail/timeline.tsx) and the `ChangeHistory`
  map+sort recompute unmemoized on every render - `useMemo`.
- `ProceedingsList` re-filters (two passes) and re-renders all rows on selection
  change; `ProceedingRow` is not `React.memo`'d - memoize + stabilize the click
  handler.

Negligible for a handful of proceedings; premature otherwise.

## F. Cross-plugin - the same `uuid` redundancy still lives elsewhere

The synthetic `uuid` we removed from katastr was inherited from the Python cores.
The same redundant per-item `uuid` (+ the `find_by_uuid` O(n) full-directory
scan) still lives in the four Python watcher plugins:

- `znamky` (`znamky_core/tracking.py`)
- `insolvence` (`insolvence_core/tracking.py`)
- `sledovana-judikatura` (in `codexis`)
- `sledovane-dokumenty` (in `codexis`)

Each addresses proceedings by `cislo_rizeni`/natural key already, so `uuid` is a
duplicate identity. `sbirka-listin` (newer Rust plugin) was written without it -
that is the clean reference. Fixing each is the same BE+FE change we did here,
per plugin.
