---
uuid: 3f8b1d6a-2c47-4e90-b15a-9d72e0c4a8f1
name: leciva
icon: icon.svg
description: Registr léčiv SÚKL (DLP). Use for Czech medicinal product lookups by drug name or SÚKL code - strength, dosage form, packaging, active substances, ATC group, dispensing type. Triggers on "lék", "léčivo", "léčivý přípravek", "SÚKL", "kód SÚKL", "účinná látka", "ATC", "příbalový leták", "SPC", "registrovaný lék", "najdi lék", "složení léku".
version: 1.5.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "SÚKL - registr léčiv"
    summary: "Vyhledávání léků v databázi SÚKL - podle názvu nebo kódu SÚKL, účinné látky, ATC."
    examplePrompts:
      - "Najdi lék Paralen - sílu, lékovou formu a jestli je vázán na předpis."
      - "Které léky registrované v ČR obsahují ibuprofen?"
      - "Kolik pacient doplácí na Ibalgin a kolik hradí pojišťovna?"
  en:
    displayName: "SÚKL - Medicines Registry"
    summary: "Look up medicines in the SÚKL database - by name or SÚKL code, active substances, ATC."
    examplePrompts:
      - "Look up the medicine Paralen - strength, dosage form and whether it needs a prescription."
      - "Which medicines registered in Czechia contain ibuprofen?"
      - "What is the patient copay for Ibalgin and how much does insurance cover?"
  sk:
    displayName: "SÚKL - register liekov"
    summary: "Vyhľadávanie liekov v databáze SÚKL - podľa názvu alebo kódu SÚKL, účinnej látky, ATC."
    examplePrompts:
      - "Nájdi liek Paralen - silu, liekovú formu a či je viazaný na predpis."
      - "Ktoré lieky registrované v ČR obsahujú ibuprofen?"
      - "Koľko pacient dopláca na Ibalgin a koľko hradí poisťovňa?"
---

# SÚKL - registr léčiv

Structured access to the Czech Medicines Registry (**Registr léčiv SÚKL**, DLP).
A single tool - **`leciva-cli`** - wraps the ai-scraper search API. There is no
local database; every command is an API call.

**IMPORTANT:** The only tool in this skill is `leciva-cli`. Do NOT call `curl` or
any other tool. Assume `leciva-cli` is installed and available in `PATH`.

**IMPORTANT:** When `leciva-cli` prints an `error:` line (e.g. an HTTP failure or
bad input), stop and report it. Do NOT guess workarounds and do NOT blindly retry.
When looking up several items, run each independently; one `error:` doesn't mean
skip the rest.

## Commands

### search
`leciva-cli search CZSUKL [OPTIONS]` - see `leciva-cli search CZSUKL --help` or
`references/search-leciva.md` for the full filter list. Output is JSON on stdout:
`{ "results": [...], "total": N, "offset": N, "limit": N }`.

**Always pipe it through `jq`** and select only the fields you need. A bare call puts
every field of every result into the context window; with a pipe, only the `jq` output
does. This matters most at `--limit 100`.

### get
`leciva-cli get <leciva://…> [--dry-run]`:

| Resource | Returns |
|---|---|
| `leciva://doc/<DOC_ID>/meta` | full medicine record (lean); add `?full=true` for all raw columns (composition, PIL/SPC links, packaging …) |
| `leciva://doc/<DOC_ID>/cena` | live price + reimbursement + patient copay (proxied from SÚKL) |
| `leciva://doc/values/<FIELD>` | distinct values + counts - `FIELD` ∈ `forma\|vydej\|stav\|atc\|typ\|indikace` |

## Workflow

1. Drug by name - keep `id` for the follow-up `get`:
   ```
   leciva-cli search CZSUKL --query "paralen" \
     | jq '.total, (.results[] | {id, nazev, sila, forma, vydej, jeDodavka})'
   ```
2. Full detail / composition - `meta` wraps the record as `{docId, source}`, so the
   raw columns (active substances, PIL/SPC links) live under `.source`. List its keys
   once, then select; do not print the whole record:
   ```
   leciva-cli get "leciva://doc/CZSUKL0000009/meta?full=true" | jq '.source | keys'
   leciva-cli get "leciva://doc/CZSUKL0000009/meta?full=true" | jq '.source | {nazev, atc}'
   ```
3. SÚKL code directly (7 digits, pass it as-is) → take the `id` → `get …/doc/<id>/meta`:
   ```
   leciva-cli search CZSUKL --kod-sukl 0000009 | jq '.results[] | {id, nazev, sila}'
   ```
4. "What contains substance X":
   ```
   leciva-cli search CZSUKL --latka "ibuprofen" --limit 100 \
     | jq '[.results[] | {nazev, sila, forma}]'
   ```
5. Price - `cena` returns `maZaznam` (whether a price is recorded), `cenaPuvodce`,
   `maxCenaLekarna` and an `uhrady[]` list carrying `uhradaPojistovny` /
   `doplatekPacienta`:
   ```
   leciva-cli get leciva://doc/CZSUKL0000009/cena \
     | jq '{maZaznam, cenaPuvodce, maxCenaLekarna, doplatek: .uhrady[0].doplatekPacienta}'
   ```

When a filter yields `null`, inspect the keys once with `jq 'keys'` (or `.source | keys`)
rather than guessing the field name.

## Result fields (search)

Each item in `results[]`: `id` (internal doc id - for `get`, never shown to the
user), `kodSukl`, `nazev`, `sila` (strength), `forma`, `vydej` (dispensing),
`atc`, `jeDodavka`, `konecPlatnostiRegistrace`.

The **`jeDodavka`** field = whether the drug is actually **on the market /
available**. Present it briefly - "available: yes / no". Do NOT recite the
technical definition (delivered in the last 6 months) unless asked.

## Presenting results

Present readably - drug, strength, form, dispensing (prescription /
over-the-counter), active substances. Do NOT dump raw JSON; the JSON is a machine
intermediate - pull only the fields you need with `jq`. Never show a raw `id`
(`CZSUKL…`), the `CZSUKL` prefix, a `leciva://` address, or an env-var name to the
user.

## Pagination & truncation

`search` returns `total` (all matches) but only `limit` results (**default 20,
max 100**).

**Pick the page size before the call, from what you need.** Re-running the same search
with a bigger `--limit` costs a whole extra round trip and leaves both outputs in
context - the first page is paid for twice.

| What you need | Call |
|---|---|
| One drug / one code (targeted) | default `--limit 20` |
| Completeness - all strengths, all packages, "is X available at all" | `--limit 100` straight away |
| Just a count | `--limit 1`, read `total`, stop |

When `total` still exceeds what you received, page with `--offset`. **Do NOT draw a
global conclusion** (e.g. "the drug is not available") from a truncated page - a drug
like Euthyrox can have dozens of variants. Deep paging is capped:
`offset + limit <= 10000`.

## Notes

- Filter values for `--forma` / `--vydej` / `--stav` are resolved Czech names, not
  raw codes - enumerate them with `get leciva://doc/values/<field>`.
- Strength is not a server-side filter. `--query "paralen 500"` matches text, not
  the numeric strength; filter strength client-side from the `sila` field if needed.
- Withdrawn registrations are excluded by default; add `--vcetne-zrusenych` to
  include them.

## Reference Files

- **`references/search-leciva.md`** - full search filter list.
