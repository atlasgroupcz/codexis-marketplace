---
uuid: 3f8b1d6a-2c47-4e90-b15a-9d72e0c4a8f1
name: leciva
icon: icon.svg
description: Registr léčiv SÚKL (DLP). Use for Czech medicinal product lookups by drug name or SÚKL code — strength, dosage form, packaging, active substances, ATC group, dispensing type. Triggers on "lék", "léčivo", "léčivý přípravek", "SÚKL", "kód SÚKL", "účinná látka", "ATC", "příbalový leták", "SPC", "registrovaný lék", "najdi lék", "složení léku".
version: 1.3.2
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

A single tool — **`leciva-cli`** — wraps the public SÚKL DLP REST API
(`https://prehledy.sukl.cz/dlp/v1`) and a local search index.

**IMPORTANT:** The only tool in this skill is `leciva-cli`. Do NOT call `curl` or any
other tool. Assume `leciva-cli` is installed and available in `PATH`.

**IMPORTANT:** When `leciva-cli` prints `ERROR:` (e.g. `HTTP 404 …`), stop and report it
to the user. Do NOT blindly retry.

## Keys

The SÚKL API cannot search by name — it only does `code → detail`. So:
1. name → code: `leciva-cli search "<text>"` (local index)
2. code → everything: `leciva-cli detail <kod>`, `leciva-cli slozeni <kod>`

The index downloads itself on first use. No API key needed.

## Commands

```bash
leciva-cli search "paralen"        # search by name → {count, shown, results:[{kodSukl, nazev, sila, doplnek}]}
leciva-cli search "paralen 500"    # name + strength together (number = strength, not package size)
leciva-cli detail 0182362          # readable detail (strength, form, dispensing, ATC, active substances, jeDodavka) — resolved names
leciva-cli detail 0182362 --all    # full record (all fields) + resolved names
leciva-cli slozeni 0182362         # active substances (names) + amounts
leciva-cli latka "ibuprofen"       # drugs containing a given active substance (also matches English, Latin, synonyms)
leciva-cli ceny 0182362            # price + insurance reimbursement + patient copay
leciva-cli dokumenty 0182362       # links to patient leaflet (PIL) and SPC
leciva-cli refresh                 # rebuild the index (also runs as a monthly automation)
leciva-cli index status            # index version / age
```

This is the whole interface. Do NOT call `curl` or any other tool and **do NOT guess DLP
endpoints** — when you need more fields, use `detail --all`, not a manual API request.

## Workflow

- Drug by name: `search "<text>"` → pick the code → `detail`/`slozeni`. You can include the
  strength in the query (`search "paralen 500"`). Text words match the name, strength and form;
  a numeric word matches the **strength** as a whole number (`50` ≠ `150`) — not the package size.
- SÚKL code directly: go straight to `detail`/`slozeni`. A SÚKL code is 7 digits; if leading
  zeros are missing they are added automatically (`254045` → `0254045`).
- "What contains substance X / which drugs have Y": `latka "<substance>"`.

## Output

On success, **stdout is always JSON** (an object, or an array for `slozeni`) with
**names** (form, route, dispensing, active substances), not raw codes. On error, a
`ERROR: …` line goes to **stderr** and the exit code is non-zero (see IMPORTANT above).
The JSON is a machine intermediate — process it with **`jq`** (pull only the fields you
need; for batches across many codes, loop and parse with `jq`, not by hand).

Then present the result to the user **readably** — drug, strength, form, dispensing
(prescription / over-the-counter), active substances — **do NOT dump raw JSON**. When you
need every field, use `detail --all`.

**Watch the cap on `search` / `latka`:** by default they return only **20 records**
(`--limit` defaults to 20). When `"truncated": true` (and `count` > `shown`), there are
more — for a drug with many strengths/packages (e.g. Euthyrox has 52 variants) raise the
limit: `leciva-cli search "euthyrox" --limit 100`. **Do NOT draw a global conclusion**
(e.g. "the drug is not available") from a truncated list — fetch all variants first.

The **`jeDodavka`** field = whether the drug is actually **on the market / available**.
Present it to the user **briefly** — "available: yes / no". Do NOT recite the technical
definition (delivered in the last 6 months) on your own; mention it only if the user asks.
