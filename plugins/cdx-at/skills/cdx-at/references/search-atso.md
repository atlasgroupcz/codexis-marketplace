# ATSO Search Filter Values

Austrian miscellaneous legal documents (Sonstige). Ministerial decrees, social insurance announcements, council of ministers protocols.

## query
Free-text search in title, short title, and content. Optional.

## application
Sub-application (exact match):
- `Erlaesse` - Ministerial Decrees (~410)
- `Avsv` - Social Insurance Announcements (~471)
- `Mrp` - Council of Ministers Protocols (~247)
- `Spg` - Security Police Act (~3)
- `Upts` - Independent Administrative Tribunals
- `Avn` - Administrative Regulations

## documentType
Document type (exact match). Values vary by application:
- Erlaesse: `Erlass`, `Rundschreiben`, etc.
- Avsv: type-specific values
- Mrp: type-specific values

## ministry
Ministry name (exact match, Erlaesse only). Example: `Bundesministerium fuer Justiz`

## dateFrom / dateTo
Date range (YYYY-MM-DD, inclusive). Matches against approvalDate (Erlaesse) or publicationDate (Avsv, others).

## sort (query param)
- `relevance` (default)
- `title`
- `date` (sorts by approval date)

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.
