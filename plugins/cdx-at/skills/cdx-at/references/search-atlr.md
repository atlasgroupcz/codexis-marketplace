# ATLR Search Filter Values

Austrian state/provincial legislation (LGBl). All 9 Austrian states.

## query
Free-text search in title, short title, and content. Optional.

## documentType
Document type (exact match):
- `Gesetz` - State law
- `Verordnung` - Regulation
- `Kundmachung` - Announcement
- `Sonstiges` - Other

## state
Austrian federal state (exact match):
- `Burgenland`
- `Kaernten`
- `Niederoesterreich`
- `Oberoesterreich`
- `Salzburg`
- `Steiermark`
- `Tirol`
- `Vorarlberg`
- `Wien`

## gazetteNumber
Exact LGBl number. Example: `LGBl. Nr. 15/2026`

## dateFrom / dateTo
Publication date range (YYYY-MM-DD, inclusive).

## sort (query param)
- `relevance` (default)
- `title`
- `date` (sorts by publication date)

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.
