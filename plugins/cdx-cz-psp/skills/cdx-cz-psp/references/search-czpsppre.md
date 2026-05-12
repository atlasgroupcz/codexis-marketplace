# CZPSPPRE Search Filter Values

Czech Parliament legislative prints (law proposals, international treaties, state budgets). Full legislative pipeline with Senate and President stages. Documents organized by election period.

## query
Free-text search in title, full title, content, and submitter. Optional.

## type
Document type (exact match):
- `Návrh zákona` - Law proposal
- `Mezinárodní smlouva` - International treaty
- `Státní rozpočet` - State budget

## electionPeriod
Election period number (integer, exact match):
- `1` (1993-1996) through `10` (2025-present)

## stateClass
Processing state class (exact match):
- `approved`
- `unapproved`
- `lightgray` (withdrawn/ended)
- `in_progress`

## pressNumber
Exact press number. Example: `69`

## submitter
Submitter name (analyzed match, AND operator). Example: `Vláda`

## currentState
Current processing state (exact match). Example: `Zákon`

## sbirkaNumber
Law collection number (exact match). Example: `459/2022 Sb.`

## eurovocDescriptor
Single EUROVOC descriptor keyword (exact match on keyword array). Example: `daň z příjmů`

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
