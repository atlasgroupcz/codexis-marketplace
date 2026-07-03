# SKNS Search Filter Values

Slovak Supreme Court (Najvyšší súd SR) decisions. ~105,000 rulings, ~1996 → present (source data lags ~1 year). The decision text lives only in the attached PDF; full-text runs over the subject matter (meritoVeci) plus the PDF's converted text.

The search body pages with `from`/`size` (not offset/limit). Multi-select filters take a JSON array.

## query
Free-text search across the subject matter (`meritoVeci`) and the decision body. Optional. Analyzed with a Czech/Slovak legal analyzer (lemmatized, §-aware).

## kolegium
Collegium (multi-select array; exact match):
- `Občianskoprávne` - Civil
- `Obchodnoprávne` - Commercial
- `Správne` - Administrative
- `Trestnoprávne` - Criminal

## chamberType
Special-chamber flag (exact match):
- `velky_senat` - Grand Chamber
- `kompetencny_senat` - Jurisdiction Chamber
- `normal` - Ordinary chamber

(Often null in the data.)

## caseNumber
Exact case number / spisová značka (keyword exact match). Format `{senatePrefix}{register}/{seq}/{year}`. Examples: `1Tdo/22/2026`, `3Cdo/145/2024`.

## decisionDateFrom / decisionDateTo
Decision date range (YYYY-MM-DD, inclusive) on the decision date.

## sort (query param)
- `relevance` (default)
- `title`
- `date`

## order (query param)
- `asc`
- `desc` (default)

## from
Integer, 0-based pagination offset. Default: 0.

## size
Integer page size, 1-100. Default: 20.

## Identifiers (result fields, not search filters)
- Display ID: `SKNS` + number, e.g. `SKNS12345`.
- ECLI: `ECLI:SK:NSSR:{year}:…` — court segment `NSSR` (distinct from the Admin. Supreme Court's `NSSSR`); may be empty on older records.
- `meritoVeci` (subject matter) is unstructured free text — the only reference field; there is no `/related` surface.
