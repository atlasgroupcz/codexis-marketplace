# SKNSS Search Filter Values

Slovak Supreme Administrative Court (Najvyšší správny súd SR) decisions. ~7,600 rulings, 2021-09 → present (the court has operated since 1 Aug 2021). Full-text runs over the legal headnote (právna veta) plus the decision PDF's converted text.

The search body pages with `from`/`size` (not offset/limit). Multi-select filters take a JSON array.

## query
Free-text search across the legal headnote (`pravnaVeta`), the Zbierka headnote (`zrPravnaVeta`), registry keywords (`heslaPreEvidenciu`), and the decision body. Optional.

## forma
Decision form (multi-select array; exact match):
- `Rozsudok` - Judgment
- `Uznesenie` - Resolution

## sud1stupna
First-instance court (multi-select array; exact match). Examples:
- `Krajský súd v Košiciach`
- `Krajský súd v Bratislave`
- `Krajský súd v Žiline`

## zr
Zbierka (collection) category flag (exact match). Example: `A` (often null).

## caseNumber
Exact NSS case number / spisová značka (keyword exact match). Format `{senate}{agenda}/{seq}/{year}`. Example: `1Vs/1/2021`.

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
- Display ID: `SKNSS` + number, e.g. `SKNSS12345`.
- ECLI: `ECLI:SK:NSSSR:{year}:{ics}.{seq}`, e.g. `ECLI:SK:NSSSR:2022:1.Vs.1.2021.1`. Court segment is always `NSSSR` — distinct from the Supreme Court's `NSSR`.
- Reference fields are unstructured free text, so there is no `/related` surface.
