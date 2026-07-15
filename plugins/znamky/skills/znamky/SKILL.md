---
uuid: 18a1a3e5-6c4d-4670-91eb-627db8b4a22d
name: znamky
icon: icon.svg
jurisdictions: [CZ, EU]
description: Ochrana a hlídání ochranných známek vůči veřejným rejstříkům ÚPV (ČR) a EUIPO (EU) přes agregátor TMview. Use for ad-hoc similarity searches (lustrace) of a word mark or a logo, and for stateful monitoring of trademarks with automatic detection of similar new filings/registrations and an AI likelihood-of-confusion (zaměnitelnost) assessment. Triggers on "ochranná známka", "ochranné známky", "trademark", "ÚPV", "EUIPO", "TMview", "zaměnitelnost", "podobná známka", "hlídej známku", "registrace značky", "kolize ochranných známek", "rešerše známky".
i18n:
  cs:
    displayName: "Hlídač ochranných známek"
    summary: "Rešerše a sledování ochranných známek (ÚPV, EUIPO) — slovní, fonetická i obrazová podobnost a posouzení zaměnitelnosti."
  en:
    displayName: "Trademark Watchdog"
    summary: "Search and monitor trademarks (ÚPV, EUIPO) — verbal, phonetic and visual similarity and likelihood-of-confusion assessment."
  sk:
    displayName: "Hliadač ochranných známok"
    summary: "Rešerš a sledovanie ochranných známok (ÚPV, EUIPO) — slovná, fonetická aj obrazová podobnosť a posúdenie zameniteľnosti."
---

# Hlídač ochranných známek (ÚPV + EUIPO)

A single tool — **`znamky-cli`** — searches the public trademark registers via **TMview**
(the EUIPO-operated common database that aggregates ÚPV "CZ", EUIPO "EM" and WIPO "WO"), scores
every hit for **verbal**, **phonetic** and **visual** similarity, and keeps a stateful watchdog for
the marks a user wants to protect.

**IMPORTANT:** For trademark data use only `znamky-cli` — do NOT call `curl` or hit TMview/ÚPV directly. `cdxctl` is the platform's automation and notification tool: use it to set up recurring monitoring or send an on-demand status e-mail, never to fetch trademark data. Assume both are installed and on `PATH`.

**IMPORTANT:** Stop only on a genuine `ERROR:` line (bad input, auth/key rejected, network, not-found) — report it, don't guess workarounds and don't blindly retry. Adding an already-tracked **word** mark (`add-text`), or removing a mark that isn't tracked, is a safe no-op (prints `OK:`) — but re-adding an already-tracked **logo** (`add-logo`) is NOT deduped and creates a duplicate. When acting on several marks, run each independently; one `ERROR:` doesn't mean skip the rest.

## Output format

- **`sledovani list`, `sledovani check`** — human-readable plain text. Summarize for the user; do
  not parse with `sed`/`grep`.
- **`sledovani add-text`, `add-logo`, `confirm`, `assess`, `remove`, `set-label`** — a single
  `OK: ...` / `ERROR: ...` line. The whole line is the status.
- **`sledovani show`, `lustrace text`, `lustrace logo`** — structured JSON. Read directly or filter
  with `jq`. Never with `sed`/`grep`.

```bash
znamky-cli lustrace text "CODEXIS" --nice 9,42 | jq '.candidates[0:5] | map({mark_text, tier, overall: .scores.overall})'
znamky-cli sledovani show <ID> | jq '.collisions | map(select(.confirmed_on == null))'
```

## Two namespaces

```bash
znamky-cli lustrace <text|logo> ...   # one-shot similarity search, no state
znamky-cli sledovani <verb> ...       # stateful monitoring of protected marks
```

---

## Decision tree — when to use what

**User asks "je tahle značka/logo volné?", "existuje něco podobného?" (one-off, no monitoring):**
→ `znamky-cli lustrace text "<NÁZEV>" [--nice 9,42] [--territory CZ,EU]`
→ `znamky-cli lustrace logo <CESTA_K_LOGU> [--text "..."] [--nice ...] [--vienna 26.4.1]`
Then present the top candidates and run the **likelihood-of-confusion assessment** (below) on the
strongest ones.

**User wants to START PROTECTING / monitoring a mark?** Trigger words: "hlídej", "sleduj",
"chraň", "dej vědět, když někdo zaregistruje podobnou".
→ `znamky-cli sledovani add-text "<NÁZEV>" --nice 9,42 --territory CZ,EU [--owner "..."] [--label "..."]`
→ `znamky-cli sledovani add-logo <CESTA> [--text "..."] --nice ... --territory ... [--vienna ...] [--label "..."]`
`--label` applies only when first adding a subject; to change an existing label use `set-label`.

`--owner` is the mark owner's name — used to recognise the user's own marks among the hits.

**User asks for a list of THEIR protected marks?**
→ `znamky-cli sledovani list`

**User asks "co je nového u <známky>?":**
1. `znamky-cli sledovani list` to find the mark id.
2. `znamky-cli sledovani check <ID>` to refresh, then `znamky-cli sledovani show <ID>` for detail.

**Never add a mark to monitoring without explicit user intent.**

**Mark collisions reviewed / rename / stop watching?**
→ `znamky-cli sledovani confirm <ID> [--all | --change <INDEX>]`
→ `znamky-cli sledovani set-label <ID> "..."`  (`""` clears)
→ `znamky-cli sledovani remove <ID>`

---

## Likelihood-of-confusion assessment (zaměnitelnost) — the AI's job

`znamky-cli` produces the **deterministic similarity triage** (verbal/phonetic via string metrics,
visual via perceptual hashing + colour, figurative via Vienna-code overlap, modulated by Nice-class
overlap). The **legal judgement is yours**. For each strong candidate (tier `high`/`medium`, or when
the user asks), assess the global "likelihood of confusion" the way EU/CZ practice does:

1. **Visual similarity** — for figurative/combined marks, OPEN the logos and compare them with your
   own vision. The watched logo is in the `show` JSON `logo_path`; a candidate's downloaded logo is
   in its `image_path` (when present). Read those image files directly.
2. **Aural similarity** — how the verbal elements sound (the CLI's `text_phonetic` score is a hint).
3. **Conceptual similarity** — meaning/idea conveyed.
4. **Similarity of goods/services** — compare the Nice classes (`class_overlap` score is a hint);
   related classes raise the risk even if not identical.
5. **Distinctiveness** of the earlier mark and the **interdependence** principle (strong similarity
   on one axis can offset weakness on another).

Then write the verdict back so it shows in the UI:

```bash
znamky-cli sledovani assess <ID> --change <INDEX> \
  --risk high --visual "..." --aural "..." --conceptual "..." \
  --goods "shodné třídy 9 a 42" --summary "Vysoké riziko záměny: ..."
```

`--risk` is one of `high|medium|low|none`. Keep each field to one or two sentences. Make clear this
is an informational assessment, not legal advice.

---

## `znamky-cli sledovani` — protected-mark monitoring

Stateful watchdog. State is stored in `~/.cdx/apps/znamky/sledovane/`. A single central cron
automation (`0 7 * * *`) runs `znamky-cli sledovani check` daily to re-search the registers and
record newly appeared similar marks.

### What is detected

- A **new similar mark** — a filing/registration whose overall similarity score is at or above the
  watched mark's threshold and that wasn't seen on a previous check.
- Each detected collision carries the full score breakdown and a `tier` (`high`/`medium`/`low`); it
  stays unconfirmed until the user (or you, after assessment) marks it reviewed.

### E-mail alerts (custom automation)

E-mail alerts are a **persistent setting** for the app — turn them on or off with `notify set`:

```bash
znamky-cli notify set --email      # turn on
znamky-cli notify set --no-email   # turn off
znamky-cli notify show             # show current status
```

When the setting is on, `znamky-cli sledovani check` e-mails a summary itself after the check,
**only if new collisions were found** (nothing is sent when there's no change). The e-mail goes
automatically to the signed-in user's address — never handle or pass an address, token or secret.

| User wants | Do |
|---|---|
| alerted when a new collision appears | `notify set --email` — `check` then e-mails on a real find |
| a status e-mail right now, collision or not | `cdxctl notification email send` (see the cdxctl skill) |

The signed-in user always gets the e-mail; `znamky-cli notify recipients add|remove|set|clear <email...>`
manage **extra** recipients on top. You don't need to add the signed-in user's own address — it's
redundant and gets dropped automatically.

When the user says e.g. *„hlídej značku CODEXIS ve třídách 9 a 42 a jednou týdně mi pošli e-mail"*:

1. Add the mark(s) (`sledovani add-text` / `add-logo`).
2. Turn on e-mail alerts: `znamky-cli notify set --email`.
3. Create **one** automation running plain `znamky-cli sledovani check` (the shipped
   `automations/check-znamky.md` template already does this). Default cadence is **weekly**
   (cron `0 7 * * 1`, Monday morning) unless the user says otherwise.
4. Briefly confirm what you set up (which marks, how often, that alerts go by e-mail).

Don't create a second parallel automation for the same marks — one plain `check` is enough; it
sends the e-mail itself when the setting is on.

---

## `znamky-cli lustrace` — one-shot similarity search

```bash
znamky-cli lustrace text "<NÁZEV>" [--nice 9,42] [--territory CZ,EU]
znamky-cli lustrace logo <CESTA> [--text "..."] [--nice 9,42] [--territory CZ,EU] [--vienna 26.4.1]
```

Returns JSON: `{ "candidates": [ { "candidate": { mark_text, mark_kind, applicant, status,
filing_date, nice_classes, vienna_codes, territory, office, url_detail, image_url }, "scores": {
text_orthographic, text_phonetic, text_combined, image_phash, image_color, vienna_overlap,
class_overlap, sign_similarity, overall }, "tier" } ], "errors": [] }`. Candidates are sorted by
`scores.overall` descending. `url_detail` links to the official TMview record.

---

## UI component

The user-facing UI is at route `/znamky` (Doplňky → Hlídač ochranných známek). It shows protected
marks (split list + detail), lets the user add a word or logo mark, refresh, review the collision
timeline with the per-signal score breakdown and your risk verdict, and open the official records.
When you make changes through `znamky-cli sledovani ...`, they appear in the UI on the next refresh.
