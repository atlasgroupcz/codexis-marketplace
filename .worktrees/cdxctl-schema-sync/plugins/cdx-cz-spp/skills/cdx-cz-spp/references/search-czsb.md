# CZSB Search Filter Values

Czech municipal regulations from sbirkapp.gov.cz. ~76,000 PP + ~130 OA records.

## query
Free-text search in title, content, legal area, legal authorization. Optional.

## druhPredpisu
Document type (exact match):

PP types (hlavniTyp=pp):
- `Obecně závazná vyhláška` - Municipal ordinance
- `Nařízení` - Regulation

OA types (hlavniTyp=oa):
- `Nález Ústavního soudu` - Constitutional Court ruling
- `Rozhodnutí o pozastavení účinnosti` - Decision on suspension
- `Smlouva` - Contract
- `Stav nebezpečí` - State of emergency

**Important:** Values require correct Czech diacritics — ASCII-stripped variants return zero results.

## publikujici
Publisher / municipality name (match query, all words must match). Example: `Statutární město Brno`

## oblastPravniUpravy
Legal area (match query, all words must match). Example: `Odpady`

## platnost
Validity status (exact match):
- `Platné` - Valid
- `Zrušeno k <date>` - Cancelled (date-specific, e.g. `Zrušeno k 16.12.2025` — not usable as a filter since each value is unique)

## cisloPredpisu
Document number (exact match). Example: `1/2026`

## ico
Municipality ICO number (exact match). Example: `44992785`

## zakonneZmocneni
Legal authorization (match query, all words must match). Free text.

## hlavniTyp
Document category (exact match):
- `pp` - Pravni predpisy (legal regulations)
- `oa` - Ostatni akty (other acts)

## datumVydaniFrom / datumVydaniTo
Issue date range (YYYY-MM-DD).

## datumZverejneniFrom / datumZverejneniTo
Publication date range (YYYY-MM-DD).

## datumUcinnostiFrom / datumUcinnostiTo
Effective date range (YYYY-MM-DD).

## sort (query param)
- `relevance` (default)
- `title`
- `date` (sorts by issue date)

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.
