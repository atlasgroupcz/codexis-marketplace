# katastr - follow-ups

Remaining candidate cleanups. The synthetic-uuid removal, the /simplify quality
pass, the shared-helper reuse (api-client, env_file, createI18n, ErrorMessage),
the dead `?settings` endpoint removal, the shared `recipients_from_body`, the
`stav` classification unification, the 3-crate -> single-crate consolidation,
reusing one process-wide ureq agent (`api_client::agent()`, with a
transport-failure retry), and folding all three ČÚZK/RÚIAN call sites onto one
`api_client::call_with_retry`, and moving every argument rule onto the clap
parser (`ArgGroup` / `requires` / `conflicts_with` / `ValueEnum`) are all
**done**; what's left is recorded below
with a rough risk so we can tell a safe change from a behavior one.

## A. Efficiency - the `check_all` hot path (deliberately deferred)

Deferred by decision: the win is real but bounded, and it must stay polite to
ČÚZK (a state API). All of these live on/near the shared `cdx-watch` check loop.

1. **Poll proceedings concurrently.** `cdx-watch/src/track.rs::check_all` maps
   `poll` over every snapshot sequentially; each `poll` is a blocking ČÚZK GET
   (30s read timeout) + a `store.save`. Wall-clock is the *sum* of round-trips
   instead of the *max*. Fix: a **bounded** thread pool (cap 4-8, not "all at
   once") so ČÚZK isn't hammered. Benefits every watcher plugin.
2. **Read the API key once per run** instead of re-reading `.env` on every
   `get()`.

## B. Not-yet-done reuse / dedup

1. **`percent_encode` still exists twice more, and has drifted.** Not
   copies of one function - functions sharing a name:

   | where | keeps literal | takes |
   | --- | --- | --- |
   | `katastr/lookup.rs` | RFC 3986 unreserved - **now the `percent-encoding` crate** | `&str` |
   | `video-analyze/main.rs` | `A-Za-z0-9-_.~` **plus `/`** | `&[u8]` |
   | `cdx-common/curl.rs::percent_encode_utf8` | every ASCII byte | `&str` |

   `video-analyze` puts a filesystem path into a query parameter
   (`?path=<encoded>`) and lets `/` through since commit `becee5bb`. Both forms
   are legal (RFC 3986 §3.4 permits a literal `/` in a query), so this is not a
   bug. It is one plugin's rule drifting inside an unrelated commit, which is
   exactly what a shared helper prevents.

   katastr now calls `percent_encoding::utf8_percent_encode`, proved
   byte-for-byte against the hand-rolled version it replaced. `video-analyze`
   can do the same: the crate has zero transitive dependencies. Note it depends
   on neither ureq nor reqwest (it shells out to `curl`), so for it this is a
   real, if tiny, new dependency - and unifying changes `video-analyze`'s wire
   format (`/` becomes `%2F`).

   Do **not** reach for `ureq`/`reqwest` here. Neither exposes an encoder; they
   only re-export `url`, whose `query_pairs_mut()` is form encoding - a space
   becomes `+`, which only a form-urlencoded parser reads back as a space.

   `cdx-common/curl.rs` looked like under-escaping and is not. `encode_query_values`
   takes an **already-formed** query string and repairs the one thing it must -
   UTF-8 - while deliberately preserving `?`, `&` and `=`, because re-encoding
   those would corrupt a locator it received correctly assembled. Free search
   text never reaches it: searches POST a JSON body. Its only input is a
   `cdx-<x>://` resource URI that the API itself produced. Leave it alone.

2. **`KatastrError` notify variants restate `NotifyError::http_status()` - and
   that is fine.** The CGI maps `InvalidRecipient` to 400 and the rest to 500,
   which is exactly what `http_status()` returns, so the statuses already agree;
   there is no bug here. Two reasons to leave it:

   - katastr's `From<NotifyError>` match is **exhaustive**, so a new variant in
     `cdx-notify` breaks the build and forces a decision. `http_status()`'s own
     `_ => 500` would swallow it silently. The duplicated rule is the stricter
     one.
   - delegating fully (`emit_error(e.http_status(), &e.to_string())`, the way
     sbirka does) would also leak the message of a 500 to the HTTP response.
     katastr answers `"Internal error"` on purpose. That is a deliberate
     difference, not drift.

   Close as won't-do unless the message-leaking question is reopened.

## C. Skipped deliberately (behavior-changing or judged not worth it)

1. **`postJson` error extraction** (FE `lib/api.ts`) - the two failure branches
   use slightly different guards; a naive merge changes which message is thrown
   in edge cases. Low value. (Now moot: `api.ts` uses the shared client, whose
   `httpError` already centralises this - only relevant if we hand-roll again.)
2. **DO NOT: run `ensure_settings()` only on notify paths.** Recorded as
   rejected - it reintroduces the migration gap we closed. Keep ensure on every
   CGI entry. Only revisit if `ensure()` ever becomes expensive.
3. ~~**`pravo_stavby::interpret_404` classifies on the rendered message.**~~
   **Watched now.** It still does, and cannot do otherwise - ČÚZK gives no code
   to key off, only the English sentence. Since the fragility cannot be removed
   it is monitored instead: `tests/pravo_stavby_marker.rs` asserts against the
   live API that a missing right is still worded the way we detect it. Ignored
   by default (one quota call); run with `-- --ignored`. Original entry kept
   below for the reasoning.

   **`pravo_stavby::interpret_404` classifies on the rendered message.** Fixing
   it properly means carrying the parsed `detail` on `KatastrError::ApiHttp`
   instead of only the formatted string. Left alone on purpose: the module
   accepts either form by design and fails to the safe side (anything
   unrecognised is an error, never "no right"), and widening the error variant
   touches every construction site plus the CGI.

4. **`ruian::search` hands back display JSON, so callers dig identifiers out of
   it.** `cmd_okoli` pulls `kod` with `.get("kod").as_u64()` and casts to
   `u32`; the next command that takes an address will copy that. A typed
   resolve (returning the picked candidate + code, with the JSON rendered for
   printing on top) would fix it. Left alone because the flat-JSON-with-`kod`
   shape is a deliberate, agreed design, not an accident.
5. ~~**Geocoding first is the wrong order for name lookups.**~~ **Done.**
   `--typ` now picks the register and asks it directly; the suggester is used
   only where a register cannot answer (no type given, or an address, whose
   stored form differs from what anyone types). Five types that had a register
   but no mapping were added. `extract_kod` is gone with it - every match is
   returned, each labelled by the parent that tells it apart.
6. ~~**`Health: 2` prints the raw integer.**~~ **Done.** The entry claimed
   mapping the enum meant guessing. It did not: every `entries[]` item already
   carries a `description` in plain words ("Connection to database is OK"), so
   `settings test` now prints those and never has to interpret the number. The
   raw status stays as a fallback if `entries` is ever missing.
7. **Independent GETs run one after another** - `okoli`'s two polygon queries,
   and the two/three diagnostic calls in `settings show`/`test`. No extra quota
   is spent, only wall-clock. Threads in a blocking one-shot CLI cost more
   complexity than the few hundred ms they would save; noted so nobody
   re-derives it.

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
