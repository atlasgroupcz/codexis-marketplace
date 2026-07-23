# sbirka-listin — follow-ups

Deferred improvements from the `/simplify` review (2026-07-20). Each is a real
step forward. They were deferred only because the blast radius reaches outside
this plugin, or the payoff is marginal right now — **not** because they're wrong.

## 1. Extract a shared frontend component-build helper

**What:** `plugins/*/components/build.sh` is duplicated nearly byte-for-byte
across `sbirka-listin`, `katastr`, `insolvence`, `znamky`, `codexis` (Docker bun
build + `sync_component_output`). Only the app dir and the preserved-assets list
differ. The install hook's `install_settings`/`install_binary`/`cleanup_own_temps`
are duplicated the same way.

**Fix:** extract `shared/scripts/component-build.sh` exposing
`build_frontend_component <app-dir> <target-dir> <preserved...>`, sourced the way
`shared/scripts/plugin-build.sh` already is. Do the same for the install-hook
helpers.

**Also fix while lifting it:** `sync_component_output`'s preserve-loop only scans
`$target_dir/assets`, but the CGI plugins pass root-level names (the CGI binary,
the dispatcher, `component.json`). Those survive today only because the cleanup
never touches the root — so the args are effectively no-ops. The shared helper
should actually model "preserve the CGI binary + dispatcher next to the SPA."

**Blast radius:** edits 5 plugins' `build.sh` (and their install hooks) and needs
each component rebuilt to verify. Its own PR.

## 2. Typed error enum for the CGI (`SbirkaError` → HTTP status)

**What:** `cgi_main.rs` hand-picks an HTTP status per call site
(`fail(500, …)`, `fail(502, …)`) because the domain calls return
`anyhow::Error` — the error category is erased, so the status is chosen by where
the error surfaced, not by what went wrong. (katastr's `katastr-cgi` maps a typed
`KatastrError` to 400/401/404/409/412/503 in one `emit_katastr_error`.)

**Fix:** define a `SbirkaError` enum with `http_status()`, return it from
`api.rs` / `http.rs` / `dokument.rs`, and replace the scattered `fail(status, e)`
with a single `emit_sbirka_error(e)`. The notify path already does this via
`cdx-notify`'s `NotifyError::http_status()`.

**Blast radius:** crate-wide error-handling refactor (~8 error sites). Contained
to this plugin. The current `fail(status, e)` picks sane statuses, so this is a
quality/precision win, not a bug fix.

## 3. Shrink the CGI (ureq instead of reqwest, or a core/cli/cgi workspace)

**What:** the CGI binary links `reqwest` + `tokio` (~5 MB) because it shares the
crate's `PortalClient` (`reqwest::blocking`). Every dashboard request spawns a
fresh per-request process carrying a tokio stack it barely uses. katastr split
`core`/`cli`/`cgi` specifically so its CGI links only `ureq` (no tokio) for a
small binary and fast cold start.

**Fix options:** (a) swap `PortalClient` from `reqwest` to `ureq` (`cdx-notify`
already pulls `ureq`, so it's in the tree), or (b) split into a Cargo workspace
with a ureq-based CGI. Option (a) is lighter.

**Payoff:** smaller binary, ~10–30 ms faster cold start per request. Not urgent —
nobody is hurting on it yet.

**Blast radius:** rewrite the HTTP layer (`http.rs`) or restructure the crate.
Contained to this plugin.

---

## Open UX decisions (need a product call, not code)

- **`sidlo` (registered address):** stored in the snapshot; shown in the detail
  header and included in the `overview` JSON (not rendered in the list row). It's
  disambiguation metadata, but it's the one field that leans into ARES's
  "company identity" territory. Options: **A** keep everywhere · **B** keep only
  in the detail, drop from `overview_rows` (recommended — the list UI doesn't use
  it) · **C** drop entirely (strict ARES boundary).
- **Vanished listiny (`zmizele_listiny`):** the Sbírka listin is append-only by
  law, so this is a defensive anomaly channel that in practice never fires. It's
  rendered as one minimal line in the change history. Keep (defensive) vs strip
  from the dashboard.
