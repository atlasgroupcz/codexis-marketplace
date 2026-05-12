# ATHI Search Filter Values

Austrian consolidated federal law norms (Bundesnormen / History). ~437,000 norm provisions with amendment chains.

## query
Free-text search in title, short title, and content. Optional.

## documentType
Document type code (exact match):
- `BG` - Bundesgesetz (Federal law)
- `V` - Verordnung (Regulation)
- And other type codes used in RIS

## abbreviation
Law abbreviation (exact match). Common examples:
- `ASVG` - Allgemeines Sozialversicherungsgesetz
- `StGB` - Strafgesetzbuch
- `B-VG` - Bundes-Verfassungsgesetz
- `ABGB` - Allgemeines buergerliches Gesetzbuch
- `StPO` - Strafprozessordnung
- `ZPO` - Zivilprozessordnung
- `AVG` - Allgemeines Verwaltungsverfahrensgesetz
- `BAO` - Bundesabgabenordnung
- `EStG` - Einkommensteuergesetz
- `GewO` - Gewerbeordnung
- `KSchG` - Konsumentenschutzgesetz
- `UStG` - Umsatzsteuergesetz

## dateFrom / dateTo
Effective date range (YYYY-MM-DD, inclusive).

## sort (query param)
- `relevance` (default)
- `title`
- `date` (sorts by effective date)

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.
