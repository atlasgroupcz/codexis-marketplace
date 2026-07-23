# SKUS Search Filter Values

Slovak Constitutional Court (Ústavný súd SR) decisions - uznesenia, nálezy, and separate dissents. ~53,000 decisions, ~1993 → present (including ~230 separate dissent records). Full-text runs over the legal headnote (právna veta) plus the decision PDF's converted text.

The search body pages with `from`/`size` (not offset/limit). Multi-select filters take a JSON array.

## query
Free-text search across the legal-headnote title/text (`clauseTitle`, `clauseText`), the keyword register (`wordRegister`), and the decision body. Optional.

## formOfDecision
Decision form (multi-select array; exact match):
- `Uznesenie` - Resolution
- `Nález` - Finding

(Separate dissent records carry an empty `formOfDecision`.)

## typeOfDecision
Decision type (multi-select array; exact match). Example: `Uznesenie o odmietnutí`.

## judgeReporter
Judge-rapporteur / sudca spravodajca (multi-select array; exact match). Example: `Robert Šorl`.

## caseNumber
Exact case number / spisová značka (keyword exact match). Format `{senate roman}. ÚS {num}/{year}`. NOT unique - one case can have several decisions. Examples: `III. ÚS 307/2026`, `I. ÚS 53/98`.

## decisionDateFrom / decisionDateTo
Decision date range (YYYY-MM-DD, inclusive) on the decision date.

## sort (query param)
- `relevance` (default) - dissent records are demoted so they never outrank the parent decision on relevance
- `title`
- `date`

## order (query param)
- `asc`
- `desc` (default)

## from
Integer, 0-based pagination offset. Default: 0.

## size
Integer page size, 1-100. Default: 20.

## Dissents (odlišné stanovisko)
A dissent is a separate record with `isDissent: true`, sharing the parent decision's `caseNumber` (there is no structured link - re-query by `caseNumber`). `differentView` carries the opinion-type label, e.g. `Odlišné stanovisko k výroku`.

## Identifiers (result fields, not search filters)
- Display ID: `SKUS` + number, e.g. `SKUS12345`.
- ECLI: `ECLI:SK:USSR:{year}:…` - court segment `USSR`; may be null on pre-ECLI records (use `caseNumber` as the primary key then).
