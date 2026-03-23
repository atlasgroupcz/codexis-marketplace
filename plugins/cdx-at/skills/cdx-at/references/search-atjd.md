# ATJD Search Filter Values

Austrian case law (Judikatur) from 7 courts. ~650,000+ decisions.

## query
Free-text search in headnote and decision text. Optional.

## application
Court code (exact match):
- `Vfgh` - Verfassungsgerichtshof / Constitutional Court (~10,600)
- `Vwgh` - Verwaltungsgerichtshof / Administrative Court (~353,800)
- `Justiz` - Justiz / Civil and Criminal Courts (~6,900)
- `Bvwg` - Bundesverwaltungsgericht / Federal Administrative Court (~237,500)
- `Lvwg` - Landesverwaltungsgerichte / State Administrative Courts (~41,000)
- `Dok` - Dokumentation / Documentation (~1,600)
- `Umse` - Umweltsenat / Environmental Senate (~390)

## documentType
Document type (exact match):
- `Entscheidungstext` - Full decision text
- `Rechtssatz` - Legal principle / headnote

## decisionType
Decision form (exact match):
- `Erkenntnis` - Judgment/finding
- `Beschluss` - Resolution/order

## caseNumber
Case file number (exact match). Example: `G 37/2024`

## ecli
ECLI identifier (exact match). Example: `ECLI:AT:VFGH:2024:G37.2024`

## state
Austrian federal state (LVwG decisions only, exact match):
- `Burgenland`
- `Kaernten`
- `Niederoesterreich`
- `Oberoesterreich`
- `Salzburg`
- `Steiermark`
- `Tirol`
- `Vorarlberg`
- `Wien`

## dateFrom / dateTo
Decision date range (YYYY-MM-DD, inclusive).

## sort (query param)
- `relevance` (default)
- `title`
- `date` (sorts by decision date)

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.
