# CZRS Search Filter Values

Czech Register of Contracts (Registr smluv, smlouvy.gov.cz) — public-sector contracts.
All filters are optional; combine freely. Use via `registr-smluv search CZRS [OPTIONS]`.

## --query / -q
Fulltext over contract subject (`predmet`) + party names (publisher + counterparties).
Typo-tolerant (fuzzy). Optional.

## --ico
IČO that matches **either** the publisher **or** any counterparty. Exact match.
Use when you don't care which side of the contract the organisation is on.

## --publisher-ico
IČO of the publisher (contracting authority) only. Exact match.

## --counterparty-ico
IČO of a counterparty (supplier) only. Exact match.

## --value-min / --value-max
Contract value **including VAT** (`hodnotaVcetneDph`), in CZK. Numeric. Bounds are inclusive.
Give either or both.

## --date-from / --date-to
Contract conclusion date (`datumUzavreni`) range. Format `YYYY-MM-DD`. Inclusive.

## --only-valid
Return only currently valid records (`platnyZaznam = true`). Flag (no value).
Omit to include superseded / invalidated versions too.

## --sort (query param)
- `relevance` (default)
- `date` (by conclusion date)

## --order (query param)
- `asc`
- `desc` (default)

## --offset / --limit
Pagination. `limit` default 20, max 100. `offset` default 0.
`offset + limit` must not exceed **10000** (deep-paging cap) — narrow with filters for more.

## Notes
- IČO is the 8-digit Czech organisation identifier (e.g. `00064581`).
- Values are in CZK including VAT.
- `--ico` is a convenience OR-filter; for a specific side use `--publisher-ico` / `--counterparty-ico`.
