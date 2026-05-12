# ATBR Search Filter Values

Austrian federal legislation (BGBl). ~18,500 documents from 2004 to present.

## query
Free-text search in title, short title, and content. Optional.

## documentType
Document type (exact match):
- `Bundesgesetz` - Federal law (Teil I)
- `Verordnung` - Regulation (Teil II)
- `Kundmachung` - Announcement (Teil III)
- `Sonstiges` - Other

## part
BGBl part (exact match):
- `Teil1` - Teil I (laws)
- `Teil2` - Teil II (regulations)
- `Teil3` - Teil III (international)

## gazetteNumber
Exact BGBl number. Example: `BGBl. II Nr. 352/2019`

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
