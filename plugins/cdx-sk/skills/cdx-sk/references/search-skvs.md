# SKVS Search Filter Values

Slovak general court decisions (okresne, krajske, etc.). ~4.6M decisions.

## query
Free-text search on decision text. Optional.

## court
Court code (exact match):
- `OSBA1` - Okresny sud Bratislava I
- `OSBA2` - Okresny sud Bratislava II
- `OSBA3` - Okresny sud Bratislava III
- `OSBA4` - Okresny sud Bratislava IV
- `OSBA5` - Okresny sud Bratislava V
- `KSBA` - Krajsky sud v Bratislave
- And many more (use courtName for full-text match)

## courtName
Full court name (exact match). Examples:
- `Okresny sud Bratislava I`
- `Krajsky sud v Bratislave`

## judge
Judge name (exact match). Example: `JUDr. Novak`

## spisovaZnacka
Case file number (exact match on keyword). Example: `1C/123/2024`

## decisionForm
Decision form:
- `Rozsudok` - Judgment
- `Uznesenie` - Resolution
- `Platobny rozkaz` - Payment order
- `Zmenkovy platobny rozkaz` - Bill of exchange payment order

## decisionNature
Decision nature:
- `Prvostupnove nenapadnute opravnymi prostriedkami` - First-instance not challenged
- `Odvolacie` - Appellate
- `Dovolacie` - Cassation

## ecli
ECLI identifier (exact match). Example: `ECLI:SK:OSBA1:2024:1234567890`

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
