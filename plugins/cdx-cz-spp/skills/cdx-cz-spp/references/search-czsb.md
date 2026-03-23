# CZSB Search Filter Values

Czech municipal regulations from sbirkapp.gov.cz. ~76,000 PP + ~130 OA records.

## query
Free-text search in title, content, legal area, legal authorization. Optional.

## druhPredpisu
Document type (exact match):

PP types (hlavniTyp=pp):
- `Obecne zavazna vyhlaska` - Municipal ordinance
- `Narizeni` - Regulation

OA types (hlavniTyp=oa):
- `Nalez Ustavniho soudu` - Constitutional Court ruling
- `Rozhodnuti o pozastaveni ucinnosti` - Decision on suspension
- `Smlouva` - Contract
- `Stav nebezpeci` - State of emergency

## publikujici
Publisher / municipality name (match query, all words must match). Example: `Statutarni mesto Brno`

## oblastPravniUpravy
Legal area (match query, all words must match). Example: `Odpady`

## platnost
Validity status (exact match):
- `Platne` - Valid
- `Neplatne` - Invalid

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
