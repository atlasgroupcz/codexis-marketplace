---
uuid: 35151977-329d-4d0c-a44b-2a1f8bf5dbb2
name: insolvence
jurisdictions: [CZ]
description: Český insolvenční rejstřík (ISIR, veřejné WS justice.cz). Use for ad-hoc insolvency lookups of companies (by IČO) and natural persons (by surname + given name + date of birth), and for tracked monitoring of subjects with automatic detection of new and changed insolvency proceedings (konkurs, oddlužení, reorganizace). Triggers on "insolvence", "insolvenční rejstřík", "ISIR", "úpadek", "konkurs", "oddlužení", "reorganizace", "je firma v insolvenci", "hlídej insolvenci", "sleduj insolvenci", "hlídač insolvencí".
i18n:
  cs:
    displayName: "Hlídač insolvencí ČR"
    summary: "Lustrace firem a osob v insolvenčním rejstříku (ISIR) a sledování změn insolvenčních řízení."
  en:
    displayName: "Czech Insolvency Watchdog"
    summary: "Look up companies and people in the Czech Insolvency Register (ISIR) and track changes to proceedings."
  sk:
    displayName: "Hliadač insolvencií ČR"
    summary: "Lustrácia firiem a osôb v insolvenčnom registri (ISIR) a sledovanie zmien insolvenčných konaní."
---

# Hlídač insolvencí ČR (ISIR)

A single tool — **`insolvence-cli`** — wraps the public ISIR web service (no API key) plus a
stateful watchdog for insolvency subjects. Source: the public ISIR SOAP service at justice.cz.

**IMPORTANT:** The only tool in this skill is `insolvence-cli`. Do NOT call `curl`, `cdxctl`, or
any other tool directly. Assume `insolvence-cli` is installed and available in `PATH`.

**IMPORTANT:** If `insolvence-cli` outputs an `ERROR:` line, stop and report it to the user.
Do not retry blindly.

## Output format

- **`sledovani list`, `sledovani check`** — human-readable plain text. Read it directly, summarize
  for the user. **Do not parse with `sed`/`grep`.**
- **`sledovani add-firma`, `add-osoba`, `confirm`, `remove`, `set-label`** — a single `OK: ...` or
  `ERROR: ...` line. The whole output is the status; no parsing needed.
- **`sledovani show`, `lustrace firma`, `lustrace osoba`** — structured JSON. Read directly or
  filter with `jq`. Never with `sed`/`grep`.

```bash
insolvence-cli lustrace firma 04863313 | jq '.rizeni[] | {spisova_znacka, stav}'
insolvence-cli sledovani show <ID> | jq '.changes | map(select(.confirmed_on == null))'
```

## Two namespaces

```bash
insolvence-cli lustrace <firma|osoba> ...   # one-shot ISIR lookup, no state
insolvence-cli sledovani <verb> ...         # stateful tracking of subjects
```

---

## Decision tree — when to use what

**The user asks "je firma X v insolvenci?" / "má osoba Y úpadek?" (one-off, no monitoring):**
→ `insolvence-cli lustrace firma <IČO>` or `insolvence-cli lustrace osoba <PŘÍJMENÍ> <JMÉNO> <RRRR-MM-DD>`

**The user wants to START MONITORING a subject?** Trigger words: "hlídej", "sleduj", "monitoruj",
"dej vědět, když", "přidej na hlídání".
→ `insolvence-cli sledovani add-firma <IČO> [--label "..."]`
→ `insolvence-cli sledovani add-osoba <PŘÍJMENÍ> <JMÉNO> <RRRR-MM-DD> [--label "..."]`

**The user asks for a list of THEIR tracked subjects?**
→ `insolvence-cli sledovani list`

**The user asks "co je nového u <subjektu>?":** check tracked state first.
1. `insolvence-cli sledovani list` to find the subject id.
2. **If tracked** → `insolvence-cli sledovani show <ID>` (rich state + change history). Optionally
   `insolvence-cli sledovani check <ID>` first to refresh from ISIR.
3. **If NOT tracked** → `insolvence-cli lustrace ...`, present the result, AND offer:
   *"Pokud chceš, můžu subjekt přidat ke sledování — budu denně hlídat nová a změněná řízení."*

**Never add a subject to tracking without explicit user intent.**

**The user wants to refresh / check now?**
→ `insolvence-cli sledovani check` (all) or `insolvence-cli sledovani check <ID>` (one)

**The user wants to mark changes read / rename / stop watching?**
→ `insolvence-cli sledovani confirm <ID> [--all | --change <INDEX>]`
→ `insolvence-cli sledovani set-label <ID> "..."`  (`""` clears)
→ `insolvence-cli sledovani remove <ID>`

`<ID>` is the subject UUID from `sledovani list`, or — for companies — the IČO.

---

## Examples

**User: "Je firma s IČO 04863313 v insolvenci?"**
```bash
insolvence-cli lustrace firma 04863313
# Present spisová značka, soud, druh (konkurs/oddlužení), stav, datum zahájení.
```

**User: "Hlídej mi insolvenci firmy 27182775 jako Dodavatel A"**
```bash
insolvence-cli sledovani add-firma 27182775 --label "Dodavatel A"
```

**User: "Sleduj insolvenci pana Jana Nováka, narozen 1.5.1980"**
```bash
insolvence-cli sledovani add-osoba "Novák" "Jan" 1980-05-01
```

**User: "Zkontroluj teď všechny moje sledované subjekty"**
```bash
insolvence-cli sledovani check
```

**User: "Označ změny u <ID> za přečtené"**
```bash
insolvence-cli sledovani confirm <ID> --all
```

---

## `insolvence-cli sledovani` — subject monitoring

Stateful watchdog. State is stored in `~/.cdx/apps/insolvence/subjekty/`. A single central cron
automation (`0 7 * * *`) runs `insolvence-cli sledovani check` daily to refresh all tracked
subjects and record changes.

### What changes are detected

- **nové řízení** — a new insolvency proceeding (spisová značka) appears for the subject
- **změna stavu** — a tracked proceeding's status changes (e.g. úpadek → konkurs → ukončeno)
- **nová událost** — reserved for event-level updates

Each detected change is stored with `confirmed_on: null` until marked read.

### Person matching caveat

Natural-person lookup matches by surname + given name + date of birth. The ISIR service only
returns a trustworthy person match when the **date of birth matches** (it otherwise falls back to
surname-only results, which this tool discards to avoid false positives). For an unambiguous match
the birth number (rodné číslo) would be required, which this tool does not collect — so a
"no insolvency" result for a person means "no proceeding found for that exact name + DOB".

---

## `insolvence-cli lustrace` — one-shot ISIR lookup

```bash
insolvence-cli lustrace firma <IČO>
insolvence-cli lustrace osoba <PŘÍJMENÍ> <JMÉNO> <RRRR-MM-DD>
```

Returns JSON: `{ "nazev"?, "rizeni": [ { spisova_znacka, soud, druh, stav, datum_zahajeni,
datum_ukonceni, aktivni, url_detail } ], "relevance" }`. The `relevance` field (1–7) reports the
ISIR match quality (1 = by rodné číslo, 2 = by IČO, 4/5 = surname + DOB matched, 6/7 = weak
surname-only — discarded for persons). `url_detail` is the official ISIR detail page for the
proceeding.

---

## UI component

The user-facing UI is at route `/insolvence` (Doplňky → Hlídač insolvencí). It shows the tracked
subjects (split list + detail), lets the user add a company or a person, refresh, and review the
change timeline. When you (AI) make tracking changes through `insolvence-cli sledovani ...`, they
appear in the UI on the next refresh — no extra step needed.
