---
uuid: 6fb435d1-5a14-4a9a-88bc-f0e3a3cd03c2
name: katastr
icon: icon.svg
jurisdictions: [CZ]
description: Český katastr nemovitostí (ČÚZK REST API KN). Use for both ad-hoc lookups (parcels, buildings, units, addresses, owners via LV, plomby/řízení signals) and tracked monitoring of cadastral proceedings (vklad/záznam status with automatic change detection). Triggers on "katastr", "parcela", "stavba", "list vlastnictví", "LV", "vklad", "záznam", "plomba", "řízení katastr", "sleduj řízení", "hlídej vklad".
i18n:
  cs:
    displayName: "Katastr nemovitostí ČR"
    summary: "Vyhledávání parcel, staveb, listů vlastnictví a sledování řízení (vklad/záznam) v katastru ČR."
  en:
    displayName: "Czech Property Registry"
    summary: "Look up parcels, buildings, title deeds and track cadastral proceedings in the Czech Property Registry."
  sk:
    displayName: "Kataster nehnuteľností ČR"
    summary: "Vyhľadávanie parciel, stavieb, listov vlastníctva a sledovanie konaní (vklad/záznam) v katastri ČR."
---

# Katastr nemovitostí ČR

A single tool — **`katastr-cli`** — wraps the entire ČÚZK REST API KN plus a stateful tracker for cadastral proceedings.

**IMPORTANT:** For cadastral data use only `katastr-cli` — do NOT call `kn`, `cdx-cli`, or `curl` (the one data exception is the RUIAN address lookup in `references/api.md`). `cdxctl` is the platform's automation tool: use it **only** to set up recurring monitoring (see *Monitoring & e-mail*), never to fetch cadastral data. Stop only on genuine `ERROR:` lines (invalid number, API key rejected/missing, network, "nebylo nalezeno v ČÚZK") — report them, don't guess workarounds. When acting on several proceedings, run each independently; one `ERROR:` doesn't mean skip the rest.

## Output Format

`katastr-cli` output comes in three shapes — consume each the right way (`grep` on JSON or `jq` on plaintext will fail):

- **`tracking list`, `tracking check`** — human-readable plain text. Read it and summarize for the user. **Do not parse with `sed`/`grep`.**
- **`tracking add/confirm/remove/set-label`, `settings set/test`** — a single `OK: ...` (success) or `ERROR: ...` (failure) line. The whole line is the status; no parsing needed.
- **`tracking show`, `api get <PATH>`** — structured JSON. Read directly or filter with `jq` (never `sed`/`grep`).

  ```bash
  katastr-cli tracking show V-123/2026-701 | jq '.stav'
  katastr-cli api get "/api/v1/Parcely/Vyhledani?..." | jq '.data[0].id'
  ```

ČÚZK errors are forwarded verbatim in the `ERROR:` line (including the API response body). Treat that message as authoritative — don't retry the same parameter; adjust or ask the user.

## Three namespaces

```bash
katastr-cli tracking <verb> ...   # stateful tracking of proceedings
katastr-cli api <verb> ...        # raw GET requests against ČÚZK — see references/api.md
katastr-cli settings <verb> ...   # API key management
```

## Decision tree — when to use what

**Raw INFO about a parcel / building / unit / address / owner / LV / plomba?**
→ `katastr-cli api get <PATH>` (one-shot lookup, no state). Endpoints and workflows
(identify parcel, "is it clean?", RUIAN address, LV, units, proceedings, code lists)
are in **`references/api.md`** — read it when doing raw lookups.

**START MONITORING a proceeding?** Trigger words: "sleduj", "hlídej", "monitoruj", "informuj mě o změnách", "začni hlídat".
→ `katastr-cli tracking add <CISLO_RIZENI> [--label "..."]`
**Never add tracking without explicit user intent** (a trigger word or an affirmative answer to the offer below).

**A list of THEIR tracked proceedings?** → `katastr-cli tracking list`

**"Co je nového / jaký je stav řízení V-XYZ?"** — ambiguous, check tracked state first:
1. `katastr-cli tracking show V-XYZ` (or `tracking list`)
2. **If tracked** → `tracking show V-XYZ` (rich state + change history); optionally `tracking check V-XYZ` first to refresh from ČÚZK.
3. **If NOT tracked** → `api get "/api/v1/Rizeni/Vyhledani?TypRizeni=V&Cislo=...&Rok=...&KodPracoviste=..."`, present it, and offer:
   *"Pokud chceš, můžu řízení přidat ke sledování — budu hlídat změny stavu a operací každý týden."*

**API key error** (`ERROR: API klíč ... není nastaven` / `... byl odmítnut`) → see **settings** below.

## `katastr-cli tracking` — proceeding monitoring

Stateful tracker. State lives in `~/.cdx/apps/katastr/rizeni/`. A central cron automation (`0 8 * * 1`) runs `katastr-cli tracking check` every Monday 8:00 to refresh all tracked proceedings.

```bash
katastr-cli tracking add V-123/2026-701 [--label "Novákovi"]       # verify in ČÚZK + save baseline
katastr-cli tracking list                                          # all tracked
katastr-cli tracking show V-123/2026-701                           # full state (JSON)
katastr-cli tracking check [V-123/2026-701] [--email] [--to ADR]   # diff vs ČÚZK (all/one); --email mails a summary every run, --to sets recipient(s)
katastr-cli tracking confirm V-123/2026-701 [--all | --change <N>] # mark changes read
katastr-cli tracking set-label V-123/2026-701 "Novákovi"           # "" clears the label
katastr-cli tracking remove V-123/2026-701
```

**Number format:** `TYPE-NUMBER/YEAR-WORKPLACE`, e.g. `V-123/2026-701` (Vklad — deposit/right transfer), `Z-100/2026-701` (Záznam — record). Supported types: `V`, `Z`, `PGP`, `PD`, `ZPV`.

**What changes are detected:**
- **stav** — overall status string (e.g. "Probíhá zpracování" → "Provedení vkladu")
- **provedené operace** — new operations appended to the timeline (Založení řízení, Zaplombování, Rozhodnutí o povolení vkladu, Provedení vkladu, Ukončení řízení, …)
- **stav úhrady** — `U` Uhrazeno, `N` Neuhrazeno, `O` Osvobozeno, `null` neuvedeno

### Monitoring & e-mail

Two separate things: the **shipped weekly refresh** (no e-mail) and **opt-in e-mail**
you set up per user.

**Shipped default — no e-mail.** `automations/check-rizeni.md` runs a plain
`tracking check` weekly to keep state fresh and the UI unread badge current. It
**does not** send e-mail. Leave it as-is unless the user asks for e-mail.

**`--email`** mails a summary of the run **every time**: the detected changes, or a
**"no new changes" heartbeat** so the user sees the watchdog is alive. Recipient is
the **signed-in user** by default; add **`--to adresa`** (repeatable) to send
elsewhere — the daemon validates the address and handles auth. You never touch a
token, SMTP or cron yourself.

**One-off "e-mail me the status now":**

```bash
katastr-cli tracking check --email
```

(Sends a summary even right after `tracking add`, when nothing has changed yet — the
heartbeat confirms it works.)

**Recurring ("sleduj a každý den/týden mi pošli e-mail"):** the recurrence *is* a
scheduled automation — one prompt run fires at most once. Set it up with `cdxctl`
(the one place you may call it):

1. Add the proceeding(s) (`tracking add`).
2. Create the e-mail automation:
   ```bash
   cdxctl automation create-command \
     --title "Katastr: e-mailové hlídání řízení" \
     --cron "0 8 * * *" \
     --command "katastr-cli tracking check --email"
   ```
   Pick the cron from the user's cadence (daily `0 8 * * *`, weekly `0 8 * * 1`, …).
   Append `--to adresa` to the `--command` for a different recipient.
3. Briefly confirm what you set up (which proceedings, how often, recipient).

## `katastr-cli api` — raw ČÚZK lookups

`katastr-cli api get "<PATH>"` — authenticated GET, JSON out. The full endpoint catalog, enums, and lookup workflows (parcels, buildings, units, LV, addresses, proceedings, code lists) live in **`references/api.md`**. Load that file when a task needs raw lookups.

## `katastr-cli settings` — API key management

```bash
katastr-cli settings show         # show configured key (masked) or "not configured"
katastr-cli settings set <KEY>    # validate against ČÚZK and persist
katastr-cli settings test         # test current key against ČÚZK
```

The key comes from the ČÚZK customer portal (free, requires Identita občana or remote-access account) and is stored in `~/.cdx/apps/katastr/.env` (shared by all `katastr-cli` subcommands and the UI). On `API klíč ... není nastaven` / `... byl odmítnut`:
1. `katastr-cli settings test` to diagnose
2. 401/403 → ask for a new key, then `katastr-cli settings set <NEW_KEY>`
3. network error → ČÚZK API is unreachable, retry later

The user can also set the key in the Katastr UI (gear icon in the header).

## UI component

The user-facing UI is at route `/katastr` (Doplňky → Katastr): a table of tracked proceedings, add/remove/refresh, and API-key management via the gear icon. Tracking changes you make through `katastr-cli tracking add/remove/confirm` appear in the UI on the next refresh — no extra step needed.

## Reference Files

- **`references/api.md`** — raw ČÚZK REST API KN: endpoint catalog, enums, response envelope, and lookup workflows (identify parcel, "is it clean?", RUIAN address → building, LV, units, proceedings, code lists). Load it when a task needs raw `api get` lookups.
