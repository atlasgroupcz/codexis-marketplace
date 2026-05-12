# SKNUS Search Filter Values

Slovak Supreme Court (NSSR) and Constitutional Court (USSR) decisions. ~2,900 decisions, 1993-2018.

## query
Free-text search across legal sentences and decision text. Optional.

## court
Court code:
- `NSSR` - Najvyssi sud Slovenskej republiky (Supreme Court, ~715 decisions)
- `USSR` - Ustavny sud Slovenskej republiky (Constitutional Court, ~2,189 decisions)

## courtName
Full court name (exact match):
- `Najvyssi sud Slovenskej republiky`
- `Ustavny sud Slovenskej republiky`

## typRozhodnutia
Decision type (Slovak):
- `Uznesenie` - Resolution
- `Nalez` - Finding (Constitutional Court)
- `Rozsudok` - Judgment

## decisionType
Alias for typRozhodnutia. Same values.

## spisovaZnacka
Case file number (exact match). Example: `3Obdo/27/2018`

## caseNumber
Alias for spisovaZnacka.

## ecli
ECLI identifier (exact match). Example: `ECLI:SK:NSSR:2017:2013200459`

## dateFrom / dateTo
Date range (YYYY-MM-DD). Filters on decision date (inclusive).

## sort (query param)
- `relevance` (default)
- `title`
- `date`

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.
