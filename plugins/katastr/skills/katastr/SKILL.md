---
uuid: 6fb435d1-5a14-4a9a-88bc-f0e3a3cd03c2
name: katastr
icon: icon.webp
jurisdictions: [CZ]
description: Český katastr nemovitostí (ČÚZK REST API KN). Use for both ad-hoc lookups (parcels, buildings, units, addresses, LV numbers, plomby/řízení signals) and tracked monitoring of cadastral proceedings (vklad/záznam status with automatic change detection). Triggers on "katastr", "parcela", "stavba", "list vlastnictví", "LV", "vklad", "záznam", "plomba", "řízení katastr", "sleduj řízení", "hlídej vklad".
i18n:
  cs:
    displayName: "Katastr nemovitostí ČR"
    summary: "Vyhledávání parcel, staveb, listů vlastnictví a sledování řízení (vklad/záznam) v katastru ČR."
    examplePrompts:
      - "Najdi v katastru dům na adrese Staroměstské náměstí 1, Praha a vypiš jeho parcely a číslo LV."
      - "Zjisti, jestli na mé parcele vázne plomba nebo na ní běží katastrální řízení."
      - "Sleduj vkladové řízení v katastru a upozorni mě, jakmile se změní jeho stav."
  en:
    displayName: "Czech Property Registry"
    summary: "Look up parcels, buildings, title deeds and track cadastral proceedings in the Czech Property Registry."
    examplePrompts:
      - "Look up the building at Staroměstské náměstí 1, Prague and list its parcels and title deed number."
      - "Check whether my parcel has a pending cadastral proceeding recorded on it."
      - "Track a cadastral deposit proceeding and alert me as soon as its status changes."
  sk:
    displayName: "Kataster nehnuteľností ČR"
    summary: "Vyhľadávanie parciel, stavieb, listov vlastníctva a sledovanie konaní (vklad/záznam) v katastri ČR."
    examplePrompts:
      - "Nájdi v katastri dom na adrese Staroměstské náměstí 1, Praha a vypíš jeho parcely a číslo LV."
      - "Zisti, či na mojej parcele viazne plomba alebo na nej beží katastrálne konanie."
      - "Sleduj vkladové konanie v katastri a upozorni ma, keď sa zmení jeho stav."
---

# Katastr nemovitostí ČR

A single tool - **`katastr-cli`** - wraps the entire ČÚZK REST API KN plus a stateful tracker for cadastral proceedings.

**IMPORTANT:** For cadastral data use only `katastr-cli` - do NOT call `kn`, `cdx-cli`, or `curl`. `cdxctl` is the platform's automation tool: use it **only** to set up recurring monitoring (see *Monitoring & e-mail*), never to fetch cadastral data. Stop only on genuine `ERROR:` lines (invalid number, API key rejected/missing, network, "nebylo nalezeno v ČÚZK") - report them, don't guess workarounds. When acting on several proceedings, run each independently; one `ERROR:` doesn't mean skip the rest.

## Output Format

`katastr-cli` output comes in three shapes - consume each the right way (`grep` on JSON or `jq` on plaintext will fail):

- **`tracking list`, `tracking check`** - human-readable plain text. Read it and summarize for the user. **Do not parse with `sed`/`grep`.**
- **`tracking add/confirm/remove/set-label`, `settings set`** - a single `OK: ...` (success) or `ERROR: ...` (failure) line. The whole line is the status; no parsing needed.
- **`tracking show`, the lookup commands (`search`, `parcela`, `okoli`, `stavba`, `jednotka`, `rizeni`, `pravo-stavby`, `sousedni`, `pracoviste`), and `api get <PATH>`** - structured JSON. Read directly or filter with `jq` (never `sed`/`grep`).
- **`settings show`, `settings test`** - an `OK:`/`ERROR:` status line (on stderr for `test`) **plus** plain-text diagnostic lines on stdout (call quota, key expiry, data-as-of date, service status, ČÚZK operational messages). Read every line and relay it to the user - the expiry warning in particular only shows up here, nowhere else.

  ```bash
  katastr-cli tracking show V-123/2026-701 | jq '.stav'
  katastr-cli parcela --ku 638790 --cislo 545 | jq '.data[0].id'
  ```

ČÚZK errors are forwarded verbatim in the `ERROR:` line (including the API response body). Treat that message as authoritative - don't retry the same parameter; adjust or ask the user.

## Three namespaces

```bash
katastr-cli tracking <verb> ...   # stateful tracking of proceedings
katastr-cli api <verb> ...        # raw GET requests against ČÚZK - see references/api.md
katastr-cli settings <verb> ...   # API key management
```

## Decision tree - when to use what

**Raw INFO about a parcel / building / unit / address / proceeding / right of superficies / neighboring parcels?**
→ Use the matching lookup command - `search`, `parcela`, `okoli`, `stavba`,
`jednotka`, `rizeni`, `pravo-stavby`, `sousedni`, `pracoviste` - not `api get`. Arguments,
one example per command, and two ready-made workflows ("is it clean?", find a
building by address) are in **`references/api.md`** - read it before doing a
raw lookup.

**START MONITORING a proceeding?** Trigger words: "sleduj", "hlídej", "monitoruj", "informuj mě o změnách", "začni hlídat".
→ `katastr-cli tracking add <CISLO_RIZENI> [--label "..."]`
`--label` applies only when first adding a subject; to change an existing label use `set-label`.
**Never add tracking without explicit user intent** (a trigger word or an affirmative answer to the offer below).

**A list of THEIR tracked proceedings?** → `katastr-cli tracking list`

**"Co je nového / jaký je stav řízení V-XYZ?"** - ambiguous, check tracked state first:
1. `katastr-cli tracking show V-XYZ` (or `tracking list`)
2. **If tracked** → `tracking show V-XYZ` (rich state + change history); optionally `tracking check V-XYZ` first to refresh from ČÚZK.
3. **If NOT tracked** → `katastr-cli rizeni <CISLO>` (e.g. `rizeni V-123/2026-701`), present it, and offer:
   *"Pokud chceš, můžu řízení přidat ke sledování - budu hlídat změny stavu a operací každý týden."*

**API key error** (`ERROR: API klíč ... není nastaven` / `... byl odmítnut`) → see **settings** below.

## `katastr-cli tracking` - proceeding monitoring

Stateful tracker. State lives in `~/.cdx/apps/katastr/rizeni/`. A central cron automation (`0 8 * * 1`) runs `katastr-cli tracking check` every Monday 8:00 to refresh all tracked proceedings.

```bash
katastr-cli tracking add V-123/2026-701 [--label "Novákovi"]       # verify in ČÚZK + save baseline
katastr-cli tracking list                                          # all tracked
katastr-cli tracking show V-123/2026-701                           # full state (JSON)
katastr-cli tracking check [V-123/2026-701]                        # diff vs ČÚZK (all/one); sends e-mail/in-app per notify settings (see below)
katastr-cli tracking confirm V-123/2026-701 [--all | --change <N>] # mark changes read
katastr-cli tracking set-label V-123/2026-701 "Novákovi"           # "" clears the label
katastr-cli tracking remove V-123/2026-701
```

**Number format:** `TYPE-NUMBER/YEAR-WORKPLACE`, e.g. `V-123/2026-701` (Vklad - deposit/right transfer), `Z-100/2026-701` (Záznam - record). Supported types: `V`, `Z`, `PGP`, `PD`, `ZPV`.

**What changes are detected:**
- **stav** - overall status string (e.g. "Probíhá zpracování" → "Provedení vkladu")
- **provedené operace** - new operations appended to the timeline (Založení řízení, Zaplombování, Rozhodnutí o povolení vkladu, Provedení vkladu, Ukončení řízení, …)
- **stav úhrady** - `U` Uhrazeno, `N` Neuhrazeno, `O` Osvobozeno, `null` neuvedeno

### Monitoring & notifications

E-mail and in-app (bell) notifications are a **persistent app setting**, not a
per-run flag on `check`:

```bash
katastr-cli notify set --email      # turn on e-mail
katastr-cli notify set --no-email   # turn off e-mail
katastr-cli notify set --inapp      # turn on the in-app bell
katastr-cli notify set --no-inapp   # turn off the in-app bell
katastr-cli notify show             # show current settings
```

`tracking check` reads this setting every run - there is no `--email`/`--to` flag on
`check` anymore. Recipient is always the **signed-in user**; you never touch a
token, SMTP or cron yourself.

**Extra recipients (optional):** the signed-in user always gets the e-mail; add
others who should get a copy. You don't need to add the signed-in user's own
address - it's redundant (they're always a recipient) and gets dropped automatically.

```bash
katastr-cli notify recipients add <email>       # add an extra recipient
katastr-cli notify recipients remove <email>    # remove one (case-insensitive)
katastr-cli notify recipients set <email...>    # replace the whole extra-recipient list
katastr-cli notify recipients clear             # remove all extras - e-mail then goes only to the signed-in user
```

- **E-mail (if on):** sent **every run**, either the detected changes or a
  **"no new changes" heartbeat** - so the user sees the watchdog is alive even on a
  quiet week.
- **In-app bell (if on):** fires **only when a proceeding actually changed** - no
  heartbeat spam in the notification center.

**Shipped default - no notifications sent by `check` unless the setting is on.**
`automations/check-rizeni.md` runs a plain `tracking check` weekly to keep state
fresh and the UI unread badge current; it sends e-mail/bell only if the user has
turned the corresponding setting on. Leave it as-is unless the user asks for
notifications.

**One-off "e-mail me / notify me now":** turn the setting on, then run
`katastr-cli tracking check` once - no flags needed:

```bash
katastr-cli notify set --email
katastr-cli tracking check
```

(Sends a summary even right after `tracking add`, when nothing has changed yet - the
e-mail heartbeat confirms it works; the bell stays quiet until a real change.)

**Recurring ("sleduj a každý den/týden mi pošli e-mail"):** the recurrence *is* a
scheduled automation - one prompt run fires at most once. Set it up with `cdxctl`
(the one place you may call it):

1. Add the proceeding(s) (`tracking add`).
2. Turn on the wanted setting(s): `katastr-cli notify set --email` and/or
   `katastr-cli notify set --inapp`.
3. Create the automation running plain `tracking check` (no flags):
   ```bash
   cdxctl automation create-command \
     --title "Katastr: hlídání řízení" \
     --cron "0 8 * * *" \
     --command "katastr-cli tracking check"
   ```
   Pick the cron from the user's cadence (daily `0 8 * * *`, weekly `0 8 * * 1`, …).
4. Briefly confirm what you set up (which proceedings, how often, which
   notification channels).

## `katastr-cli` - lookups

`search`, `parcela`, `okoli` (nearby parcels/buildings), `stavba`,
`jednotka`, `rizeni`, `pravo-stavby`, `sousedni`, `pracoviste`. Arguments and
examples are in **`references/api.md`**.

`katastr-cli api get "<PATH>"` is the escape hatch for endpoints none of the
commands above cover. Don't use it when a command exists.

## `katastr-cli settings` - API key management

```bash
katastr-cli settings show         # show configured key (masked) or "not configured"
katastr-cli settings set <KEY>    # validate against ČÚZK and persist
katastr-cli settings test         # test current key against ČÚZK
```

`show` also reports the account's call quota and the key's expiry date; `test`
also reports data-as-of date and service health. Both are best-effort on top
of the local key status/validation - relay whatever they print, including any
ČÚZK operational message (e.g. an approaching key expiry).

The key comes from the ČÚZK customer portal (free, requires Identita občana or remote-access account) and is stored in `~/.cdx/apps/katastr/.env` (shared by all `katastr-cli` subcommands and the UI). On `API klíč ... není nastaven` / `... byl odmítnut`:
1. `katastr-cli settings test` to diagnose
2. 401/403 → ask for a new key, then `katastr-cli settings set <NEW_KEY>`
3. network error → ČÚZK API is unreachable, retry later

The user can also set the key in the Katastr UI (gear icon in the header).

## UI component

The user-facing UI is at route `/katastr` (Doplňky → Katastr): a table of tracked proceedings, add/remove/refresh, and API-key management via the gear icon. Tracking changes you make through `katastr-cli tracking add/remove/confirm` appear in the UI on the next refresh - no extra step needed.

## Reference Files

- **`references/api.md`** - the lookup commands (`search`, `parcela`, `okoli`, `stavba`, `jednotka`, `rizeni`, `pravo-stavby`, `sousedni`, `pracoviste`), the response envelope, identifiers, and two workflows ("is it clean?", find a building by address). `api get` is documented last, as the escape hatch. Load it when a task needs a raw lookup.
