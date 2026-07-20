---
uuid: af41bf2b-3172-48e3-a47f-f2b2aa5e5a67
name: register-zmluv
icon: icon.svg
description: This skill should be invoked whenever the user needs Slovak public-sector contracts from the Central Register of Contracts (Centrálny register zmlúv, crz.gov.sk) — search by subject, contracting parties, IČO, contract value, or contract date. Triggers on "zmluva", "register zmlúv", "centrálny register zmlúv", "crz.gov.sk", "verejná zmluva", "dodávateľ", "objednávateľ", "obstarávateľ", "hodnota zmluvy", "kto podpísal zmluvu s", "slovenské zmluvy".
version: 1.2.0
jurisdictions: [SK]
i18n:
  cs:
    displayName: "Register zmlúv SR"
    summary: "Vyhledávání slovenských veřejných smluv v Centrálním registru zmlúv (crz.gov.sk) — podle předmětu, stran, IČO, hodnoty a data uzavření."
  en:
    displayName: "Slovak Register of Contracts"
    summary: "Search public-sector contracts in the Slovak Central Register of Contracts (crz.gov.sk) — by subject, parties, IČO, value, and contract date."
  sk:
    displayName: "Centrálny register zmlúv"
    summary: "Vyhľadávanie verejných zmlúv v Centrálnom registri zmlúv (crz.gov.sk) — podľa predmetu, strán, IČO, hodnoty a dátumu uzavretia."
---

# Slovak Register of Contracts (register-zmluv)

Structured access to the Slovak Central Register of Contracts (**Centrálny register zmlúv**,
published at crz.gov.sk) — public-sector contracts with their subject (predmet), registration
number (nazov), contracting parties, value, and dates. Covers contracts published since
2011-01-01. Single data source: `SKCRZ`.

A single tool — **`register-zmluv`** — wraps the search API. Do NOT call `curl` or any other
tool. Assume `register-zmluv` is installed and available in `PATH`.

**IMPORTANT:** When `register-zmluv` prints an `error:` line (e.g. an HTTP failure), stop and
report it to the user. Do NOT blindly retry.

## Commands

### search
Search contracts: `register-zmluv search SKCRZ [OPTIONS]`

Use `register-zmluv search SKCRZ --help` for the full filter list. Output is JSON on stdout:
`{ "results": [...], "total": N, "offset": N, "limit": N }`.

**Always pipe it through `jq`** and select only the fields you need. A bare call puts
every field of every result into the context window; with a pipe, only the `jq` output
does. **Always keep `url` in the selection** — it is the citable link, and a filter that
drops it leaves you unable to cite the contract.

### get
Fetch a contract's full record: `register-zmluv get register-zmluv://doc/<DOC_ID>/meta [--dry-run]`

`meta` is the only document resource — it returns the whole contract record.

## Workflow

1. Find contracts — keep `url` for citing and `docId` for the follow-up `get`. Select
   `predmet` as the headline, `nazov` only as a secondary reference:
   ```
   register-zmluv search SKCRZ --query "poskytovanie služieb" \
     | jq '.total, (.results[] | {docId, url, predmet, nazov, party1Name, party2Name, sumaSpolu, datum})'
   ```
2. Just how many there are — one call, read `total`, stop:
   ```
   register-zmluv search SKCRZ --query "poskytovanie služieb" --limit 1 | jq '.total'
   ```
3. Need the full record — `meta` wraps it as `{docId, contract}`, so the raw columns
   are under `.contract`. List the keys once, then select:
   ```
   register-zmluv get register-zmluv://doc/SKCRZ123456/meta | jq '.contract | keys'
   ```

When a filter yields `null`, inspect the keys with `jq 'keys'` (or `.contract | keys`)
rather than guessing the field name.

## Result fields (search)

Each item in `results[]`:

| field | meaning |
|---|---|
| `docId` | internal ID (e.g. `SKCRZ123456`) — for `get`; never shown to the user |
| `url` | public detail page on crz.gov.sk — **this is the citable source link** |
| `predmet` | contract subject — the descriptive one. **Use this as the headline.** |
| `nazov` | the publisher's registration number (e.g. `073/2024`), NOT a descriptive title |
| `party1Name` / `party2Name` | the two contracting parties |
| `partyIcos[]` | IČOs of both parties (flat list — the side is not distinguished) |
| `sumaSpolu` | total contract value (**EUR**) |
| `datum` | contract date |
| `datumZverejnene` | publication date |
| `attachmentCount` | number of document attachments |

## Metadata-only source

This register is **metadata-only**. We host no contract PDFs — the official documents live on
the contract's own detail page on crz.gov.sk. To give the user the document, link the `url`;
never claim to have the contract text itself.

## User-Facing Output Rules

The raw identifiers from the API are for building CLI calls only — they must never leak into
user-visible text.

### Link Format

Cite each contract with a markdown link to its **`url`** (the public crz.gov.sk detail page):
`[<predmet> — <party1Name>](https://www.crz.gov.sk/zmluva/...)`. That page links the official
contract PDF and its attachments, so the user can open the document from there.

Link the **`predmet`**, never the `nazov`. `nazov` is the publisher's internal registration
number (`073/2024`), so a link titled with it tells the user nothing. Mention `nazov` only as a
secondary reference ("ev. č. 073/2024") when it helps identify the contract.

A `register-zmluv://` link is NOT user-facing — it is an internal address you dereference with
`register-zmluv get register-zmluv://…`. Never show a `register-zmluv://` link or a raw `docId`
to the user.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `SKCRZ123456`)
- Raw source prefix (`SKCRZ`)
- `register-zmluv://` internal addresses
- Environment variable names (e.g., `$CODEXIS_PLUGIN_REGISTER_ZMLUV_API_URL`)
- HTML tags — use markdown links only

### Human-Readable Source Name

When referring to the source in prose, match the user's language:

| Code (internal) | Slovak | Czech | English |
|---|---|---|---|
| `SKCRZ` | Centrálny register zmlúv | Centrální registr smluv SR | Slovak Register of Contracts |

### Presenting results

Present contracts **readably** — subject, parties (who with whom), value, contract date — and
link the `predmet` to the contract's `url`. Do NOT dump raw JSON. The JSON is a machine
intermediate; pull only the fields you need with `jq`.

**Correct:**
```
[Zmluva poskytovania služieb spoločnosti Microsoft — Obec Banka](https://www.crz.gov.sk/zmluva/8301397)
— Microsoft Ireland Operations Limited, ev. č. 073/2024, uzavreté 2024-08-30, 3 prílohy
```

**Incorrect:**
```
[073/2024 — Obec Banka](...) — wrong, `nazov` is a registration number, not a title
SKCRZ123456 — wrong, raw document ID
register-zmluv://doc/SKCRZ123456/meta — wrong, internal address
```

## Pagination & truncation

- `search` returns `total` (all matches) but only `limit` results (**default 20, max 100**). When
  `total` is larger than what you received, there are more contracts.
- **Pick the page size before the call.** Re-running the same search with a bigger `--limit` costs
  an extra round trip and leaves both outputs in context — the first page is paid for twice.

  | What you need | Call |
  |---|---|
  | A specific contract, or the top few matches | default `--limit 20` |
  | Completeness — every contract of a party, all contracts in a range | `--limit 100` straight away |
  | Just a count | `--limit 1`, read `total`, stop |

- When `total` still exceeds what you received, page with `--offset`. **Do NOT draw a global
  conclusion** (e.g. "there is only one such contract") from a truncated first page.
- Deep paging is capped: `offset + limit` must not exceed **10000**. For broader coverage, narrow
  the query with filters instead.
- For a count, read `total` from one `--limit 1` search — never page through to count, and never
  loop `/meta` over many records. The register holds millions of contracts since 2011; large dumps
  blow up the context window.

## Source data quirks

Data mirrors the Central Register of Contracts 1:1, including publishers' mistakes:

- **Missing dates.** The source uses a `0000-00-00` placeholder for "no date"; it is indexed as
  null, so such contracts drop out of `--date-from` / `--date-to` ranges entirely. A date filter
  therefore silently excludes undated records — don't treat a filtered `total` as the whole truth.
- **Implausible dates.** `datum` is the contract date as the publisher typed it. Expect the odd
  nonsense year (publisher typos). `--sort date --order desc` puts those at the top, so a naive
  "newest" query can return garbage. Sanity-check dates when surfacing the most recent contracts;
  optionally cap with `--date-to <today>`.
- **`ico` does not distinguish the side.** `partyIcos` is a flat list of both parties, so `--ico`
  matches a contract where the organisation is on **either** side. There is no publisher-only or
  supplier-only filter — unlike the Czech register. If the user asks "what did X buy", you cannot
  filter that server-side; inspect `party1Name` / `party2Name` per result.
- **`sumaSpolu: 0` does not mean the contract was free.** A zero (or missing) value is common —
  framework agreements, contracts priced per unit, and plain publisher omissions all show 0. Say
  what the register records ("evidovaná hodnota 0 €"), never conclude the performance was
  gratuitous. The real figure may only be in the PDF on the contract's page.
- **`rezort` ids are opaque.** The value is a numeric id string (e.g. `114573`) with no lookup
  table and no endpoint to enumerate it. Despite the name it is not restricted to ministries — it
  can be a municipality. Only use `--rezort` when the user supplies a concrete id; do not guess.
- **Amendments are separate records.** A dodatok is its own contract record with no clean filter,
  so an exact "unique contracts" count is NOT cheaply available. Report `total`, note that
  amendments are included, and offer to narrow — do NOT page through every record to classify.

## Reference Files

- **`references/search-skcrz.md`** — full search filter list: fulltext, IČO, rezort, value
  range, date range, sorting.
