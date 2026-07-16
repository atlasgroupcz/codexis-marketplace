---
uuid: 3f8b1d6a-2c47-4e90-b15a-9d72e0c4a8f1
name: leciva
icon: icon.svg
description: Registr léčiv SÚKL (DLP). Use for Czech medicinal product lookups by drug name or SÚKL code — strength, dosage form, packaging, active substances, ATC group, dispensing type. Triggers on "lék", "léčivo", "léčivý přípravek", "SÚKL", "kód SÚKL", "účinná látka", "ATC", "příbalový leták", "SPC", "registrovaný lék", "najdi lék", "složení léku".
version: 1.4.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "SÚKL — registr léčiv"
    summary: "Vyhledávání léků v databázi SÚKL — podle názvu nebo kódu SÚKL, účinné látky, ATC."
  en:
    displayName: "SÚKL — Medicines Registry"
    summary: "Look up medicines in the SÚKL database — by name or SÚKL code, active substances, ATC."
  sk:
    displayName: "SÚKL — register liekov"
    summary: "Vyhľadávanie liekov v databáze SÚKL — podľa názvu alebo kódu SÚKL, účinnej látky, ATC."
---

# SÚKL — registr léčiv

Structured access to the Czech Medicines Registry (**Registr léčiv SÚKL**, DLP).
A single tool — **`leciva-cli`** — wraps the ai-scraper search API. There is no
local database; every command is an API call.

**IMPORTANT:** The only tool in this skill is `leciva-cli`. Do NOT call `curl` or
any other tool. Assume `leciva-cli` is installed and available in `PATH`.

**IMPORTANT:** When `leciva-cli` prints an `error:` line (e.g. an HTTP failure or
bad input), stop and report it. Do NOT guess workarounds and do NOT blindly retry.
When looking up several items, run each independently; one `error:` doesn't mean
skip the rest.

## Commands

### search
`leciva-cli search CZSUKL [OPTIONS]` — see `leciva-cli search CZSUKL --help` or
`references/search-leciva.md` for the full filter list. Output is JSON on stdout:
`{ "results": [...], "total": N, "offset": N, "limit": N }`. Process it with `jq`.

### get
`leciva-cli get <leciva://…> [--dry-run]`:

| Resource | Returns |
|---|---|
| `leciva://doc/<DOC_ID>/meta` | full medicine record (lean); add `?full=true` for all raw columns (composition, PIL/SPC links, packaging …) |
| `leciva://doc/<DOC_ID>/cena` | live price + reimbursement + patient copay (proxied from SÚKL) |
| `leciva://doc/values/<FIELD>` | distinct values + counts — `FIELD` ∈ `forma\|vydej\|stav\|atc\|typ\|indikace` |

## Workflow

1. Drug by name: `leciva-cli search CZSUKL --query "<text>"` (+ filters). Each
   result carries `id` (e.g. `CZSUKL0000009`), `kodSukl`, `nazev`, `sila`,
   `forma`, `vydej`, `atc`, `jeDodavka`.
2. Full detail / composition: `leciva-cli get "leciva://doc/<id>/meta?full=true"`
   — the raw record holds active substances (složení) and PIL/SPC document links.
3. SÚKL code directly: `leciva-cli search CZSUKL --kod-sukl <code>` → take the
   `id` → `get …/doc/<id>/meta`. A SÚKL code is 7 digits; pass it as-is.
4. "What contains substance X": `leciva-cli search CZSUKL --latka "<substance>"`.
5. Price: `leciva-cli get leciva://doc/<id>/cena`.

## Result fields (search)

Each item in `results[]`: `id` (internal doc id — for `get`, never shown to the
user), `kodSukl`, `nazev`, `sila` (strength), `forma`, `vydej` (dispensing),
`atc`, `jeDodavka`, `konecPlatnostiRegistrace`.

The **`jeDodavka`** field = whether the drug is actually **on the market /
available**. Present it briefly — "available: yes / no". Do NOT recite the
technical definition (delivered in the last 6 months) unless asked.

## Presenting results

Present readably — drug, strength, form, dispensing (prescription /
over-the-counter), active substances. Do NOT dump raw JSON; the JSON is a machine
intermediate — pull only the fields you need with `jq`. Never show a raw `id`
(`CZSUKL…`), the `CZSUKL` prefix, a `leciva://` address, or an env-var name to the
user.

## Pagination & truncation

`search` returns `total` (all matches) but only `limit` results (**default 20,
max 100**). When `total` exceeds what you received there are more — page with
`--offset` or raise `--limit`. **Do NOT draw a global conclusion** (e.g. "the drug
is not available") from a truncated page. A drug with many strengths/packages
(e.g. Euthyrox) can have dozens of variants — raise the limit before concluding.
Deep paging is capped: `offset + limit <= 10000`.

## Notes

- Filter values for `--forma` / `--vydej` / `--stav` are resolved Czech names, not
  raw codes — enumerate them with `get leciva://doc/values/<field>`.
- Strength is not a server-side filter. `--query "paralen 500"` matches text, not
  the numeric strength; filter strength client-side from the `sila` field if needed.
- Withdrawn registrations are excluded by default; add `--vcetne-zrusenych` to
  include them.

## Reference Files

- **`references/search-leciva.md`** — full search filter list.
