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

**IMPORTANT:** The only tool in this skill is `znamky-cli`. Do NOT call `curl` or hit TMview/ÚPV
directly. Assume `znamky-cli` is installed and on `PATH`.

**IMPORTANT:** If `znamky-cli` prints an `ERROR:` line, stop and report it. Do not retry blindly.

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

### E-mailová upozornění (automatizace na míru)

When the user wants e-mail alerts, run the check with `--email`:

```bash
znamky-cli sledovani check --email
```

`--email` sends a summary **only if new collisions were found**. It goes to the signed-in user's
address automatically — never handle an address, token or secret.

When the user says e.g. *„hlídej značku CODEXIS ve třídách 9 a 42 a jednou týdně mi pošli e-mail"*:

1. Add the mark(s) (`sledovani add-text` / `add-logo`).
2. Create an **automation** running `znamky-cli sledovani check --email`. Default cadence is
   **weekly** (cron `0 7 * * 1`, Monday morning) unless the user says otherwise.
3. Briefly confirm what you set up (which marks, how often, that alerts go by e-mail).

That automation does its own check — don't also schedule a plain `check` for the same marks, or the
earlier run would consume the changes and the e-mail wouldn't arrive.

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
