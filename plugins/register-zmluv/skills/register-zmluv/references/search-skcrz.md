# SKCRZ Search Filter Values

Slovak Central Register of Contracts (Centrálny register zmlúv, crz.gov.sk) — public-sector
contracts published since 2011-01-01. All filters are optional; combine freely.
Use via `register-zmluv search SKCRZ [OPTIONS]`.

## --query / -q
Fulltext over `nazov` + subject (`predmet`) + party names. Typo-tolerant (fuzzy). Optional.

Note `nazov` is the publisher's registration number (e.g. `073/2024`), not a descriptive title —
`predmet` carries the description. Display `predmet`, not `nazov`.

## --ico
IČO that matches **either** contract party. Exact match.
The register stores both parties' IČOs in one flat list, so there is no publisher-only or
supplier-only variant — this filter cannot tell you which side the organisation was on.

## --rezort
Publishing sector/ministry id. Exact match.
Opaque numeric id string (e.g. `114573`) — there is no lookup table and no endpoint to list the
values, and despite the name it is not limited to ministries (a municipality has one too).
Only use when the user supplies a concrete id.

## --value-min / --value-max
Total contract value (`sumaSpolu`), in **EUR**. Numeric. Bounds are inclusive.
Give either or both.

## --date-from / --date-to
Contract date (`datum`) range. Format `YYYY-MM-DD`. Inclusive.
Contracts with no date (source placeholder `0000-00-00` → null) are excluded by any date filter.

## --sort (query param)
- `relevance` (default)
- `date` (by contract date)

## --order (query param)
- `asc`
- `desc` (default)

## --offset / --limit
Pagination. `limit` default 20, max 100. `offset` default 0.
`offset + limit` must not exceed **10000** (deep-paging cap) — narrow with filters for more.

## Notes
- IČO is the 8-digit Slovak organisation identifier (e.g. `00151513`).
- Values are in EUR. `sumaSpolu: 0` is common and does NOT mean the contract was free — report it
  as the registered value, don't infer gratuity.
- Metadata-only: the API returns no contract text or PDFs. Each result's `url` is the public
  crz.gov.sk detail page, which links the official document and its attachments.

## Examples

### Basic fulltext search
```bash
register-zmluv search SKCRZ --query "údržba" --limit 5 \
  | jq '.results[] | {predmet, nazov, party1Name, party2Name, sumaSpolu, datum, url}'
```

### All contracts of one organisation (either side)
```bash
register-zmluv search SKCRZ --ico 00151513 --limit 10 \
  | jq '.results[] | {predmet, party1Name, party2Name, sumaSpolu, datum}'
```

### Date-range search
```bash
register-zmluv search SKCRZ --query "nájomná zmluva" \
  --date-from 2024-01-01 --date-to 2024-12-31 --limit 10 \
  | jq '.results[] | {predmet, datum, sumaSpolu}'
```

### High-value contracts, newest first
```bash
register-zmluv search SKCRZ --value-min 1000000 --sort date --order desc --limit 10 \
  | jq '.results[] | {predmet, sumaSpolu, datum, url}'
```

### Count only (never page through to count)
```bash
register-zmluv search SKCRZ --query "stavebné práce" --limit 1 | jq '.total'
```

## Working with SKCRZ Documents

```bash
DOC_ID="SKCRZ123456"

# Full contract record (all indexed fields, raw from crz.gov.sk)
register-zmluv get "register-zmluv://doc/${DOC_ID}/meta" | jq '.contract'
```

`meta` is the only document resource — this register is metadata-only, so there is no `/text`,
`/toc`, `/versions`, or `/parts` endpoint. For the official document, open the contract's `url`.
