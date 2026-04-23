# CZPSPDOK Search Filter Values

Czech Parliament non-legislative documents (reports, interpellations, EU consent requests). Documents organized by election period.

## query
Free-text search in title, full title, content, and author. Optional.

## documentType
Document type (exact match):
- `Zpráva a jiné` - Reports (budget execution, NKU, CNB reports)
- `Písemná interpelace` - Written interpellations (MP questions to government)
- `Předchozí souhlas s rozhodnutím orgánů EU` - EU document consent requests
- `Jiné podklady` - Other supporting documents

## electionPeriod
Election period number (integer, exact match):
- `1` (1993-1996) through `10` (2025-present)

## stateClass
Processing state class (exact match):
- `approved`
- `unapproved`
- `in_progress`

## pressNumber
Exact press number. Example: `142`, `66-E` (EU docs have `-E` suffix).

## author
Author name (analyzed match, AND operator). Example: `Vláda`

## currentState
Current processing state (exact match). Example: `Zpráva`

## addressee
Addressee name (analyzed match, AND operator). Interpellations only. Example: `Babiš`

## submissionDateFrom / submissionDateTo
Submission date range (YYYY-MM-DD, inclusive).

## sort (query param)
- `relevance` (default)
- `title`
- `date` (sorts by submission date)
- `pressNumber`

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.
