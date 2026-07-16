---
uuid: 9c2f1b7a-4e83-4d16-b0a5-7f3c6e21d904
name: registr-smluv
icon: icon.svg
description: This skill should be invoked whenever the user needs Czech public-sector contracts from the Register of Contracts (Registr smluv, smlouvy.gov.cz) — search by subject, contracting parties, IČO, contract value, or conclusion date. Triggers on "smlouva", "registr smluv", "veřejná smlouva", "smlouvy.gov.cz", "dodavatel", "objednatel", "zadavatel", "hodnota smlouvy", "kdo podepsal smlouvu s".
version: 1.1.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "Registr smluv ČR"
    summary: "Vyhledávání veřejných smluv v Registru smluv (smlouvy.gov.cz) — podle předmětu, stran, IČO, hodnoty a data uzavření."
  en:
    displayName: "Czech Register of Contracts"
    summary: "Search public-sector contracts in the Czech Register of Contracts (smlouvy.gov.cz) — by subject, parties, IČO, value, and conclusion date."
  sk:
    displayName: "Register zmlúv ČR"
    summary: "Vyhľadávanie verejných zmlúv v Registri zmlúv (smlouvy.gov.cz) — podľa predmetu, strán, IČO, hodnoty a dátumu uzavretia."
---

# Czech Register of Contracts (registr-smluv)

Structured access to the Czech Register of Contracts (**Registr smluv**, published at
smlouvy.gov.cz) — public-sector contracts with their subject (předmět), contracting parties,
value, and dates. Single data source: `CZRS`.

A single tool — **`registr-smluv`** — wraps the search API. Do NOT call `curl` or any other
tool. Assume `registr-smluv` is installed and available in `PATH`.

**IMPORTANT:** When `registr-smluv` prints an `error:` line (e.g. an HTTP failure), stop and
report it to the user. Do NOT blindly retry.

## Commands

### search
Search contracts: `registr-smluv search CZRS [OPTIONS]`

Use `registr-smluv search CZRS --help` for the full filter list. Output is JSON on stdout:
`{ "results": [...], "total": N, "offset": N, "limit": N }`. Process it with `jq`.

### get
Fetch a contract's full record: `registr-smluv get registr-smluv://doc/<DOC_ID>/meta [--dry-run]`

`meta` is the only document resource — it returns the whole contract record.

## Workflow

1. Find contracts: `registr-smluv search CZRS --query "<text>"` (+ filters). Each result carries
   a `docId` (e.g. `CZRS34840741`) and a `url` (its public detail page).
2. Need the full record: `registr-smluv get registr-smluv://doc/<docId>/meta`.

## Result fields (search)

Each item in `results[]`:

| field | meaning |
|---|---|
| `docId` | internal ID (e.g. `CZRS34840741`) — for `get`; never shown to the user |
| `url` | public detail page on smlouvy.gov.cz — **this is the citable source link** |
| `predmet` | contract subject |
| `publisherName` / `publisherIco` | contracting authority (name + IČO) |
| `counterpartyNames[]` / `counterpartyIcos[]` | counterparties (suppliers) |
| `hodnotaVcetneDph` | contract value incl. VAT (CZK) |
| `datumUzavreni` | conclusion date |
| `casZverejneni` | publication time |
| `platnyZaznam` | whether this is the currently valid version |
| `attachmentCount` | number of document attachments |

## User-Facing Output Rules

The raw identifiers from the API are for building CLI calls only — they must never leak into
user-visible text.

### Link Format

Cite each contract with a markdown link to its **`url`** (the public smlouvy.gov.cz detail
page): `[<předmět> — <publisherName>](https://smlouvy.gov.cz/...)`. That page links the official
contract PDF, so the user can open the document from there.

A `registr-smluv://` link is NOT user-facing — it is an internal address you dereference with
`registr-smluv get registr-smluv://…`. Never show a `registr-smluv://` link or a raw `docId` to
the user.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `CZRS34840741`)
- Raw source prefix (`CZRS`)
- `registr-smluv://` internal addresses
- Environment variable names (e.g., `$CODEXIS_PLUGIN_REGISTR_SMLUV_API_URL`)
- HTML tags — use markdown links only

### Human-Readable Source Name

When referring to the source in prose, match the user's language:

| Code (internal) | Czech | English |
|---|---|---|
| `CZRS` | Registr smluv | Czech Register of Contracts |

### Presenting results

Present contracts **readably** — subject, parties (who with whom), value, conclusion date — and
link the title to the contract's `url`. Do NOT dump raw JSON. The JSON is a machine intermediate;
pull only the fields you need with `jq`.

**Correct:**
```
[Rekonstrukce silnice II/150 — Ředitelství silnic a dálnic ČR](https://smlouvy.gov.cz/smlouva/12345678)
— dodavatel EUROVIA CS, 12,4 mil. Kč vč. DPH, uzavřeno 2024-03-11
```

**Incorrect:**
```
CZRS34840741 — wrong, raw document ID
registr-smluv://doc/CZRS34840741/meta — wrong, internal address
```

## Pagination & truncation

- `search` returns `total` (all matches) but only `limit` results (**default 20, max 100**). When
  `total` is larger than what you received, there are more contracts.
- Page with `--offset`, or raise `--limit` (up to 100). **Do NOT draw a global conclusion** (e.g.
  "there is only one such contract") from a truncated first page.
- Deep paging is capped: `offset + limit` must not exceed **10000**. For broader coverage, narrow
  the query with filters instead.
- For a count, read `total` from one `--limit 1` search — never page through to count, and never
  loop `/meta` over many records. Totals reach millions; large dumps blow up the context window.

## Source data quirks

Data mirrors the Register of Contracts 1:1, including publishers' mistakes:

- **Implausible dates.** `datumUzavreni` is the signing date. A few records carry clearly wrong
  years (e.g. `9917`, `3021`) — publisher typos. `--sort date --order desc` puts them at the top,
  so a naive "newest" query returns garbage. When surfacing the most recent contracts, sanity-check
  the dates and ignore absurd years; optionally cap with `--date-to <today>` (a hard cap may drop
  the rare, legitimately post-dated contract).
- **`--only-valid` is not a date check.** `platnyZaznam` marks the current *version* of a contract,
  not a correct date — a garbage-dated record can still be valid. Combine `--only-valid` with a
  date sanity check for "newest real contracts".
- **A `total` counts records, not contracts — don't brute-force "unique".** `--only-valid`'s
  `total` (one `--limit 1` search) already counts distinct records with versions collapsed. But an
  amendment (dodatek) is a separate record with no clean filter, so an exact "unique / excluding
  amendments" count is NOT cheaply available. Report the `--only-valid` total, note that amendments
  are included, and offer to narrow — do NOT page through every record to classify titles.

## Reference Files

- **`references/search-smlouvy.md`** — full search filter list: fulltext, IČO filters, value
  range, date range, validity, sorting.
