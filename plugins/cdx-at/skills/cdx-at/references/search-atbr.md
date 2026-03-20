# Austrian Federal Legislation Search (ATBR — Bundesrecht)

Federal legislation as published in the Bundesgesetzblatt (BGBl). Covers laws, decrees, and regulations published in BGBl Teil I, II, and III.

## Search Request

```bash
cdx-at -s -X POST "cdx-at://search/ATBR" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "Datenschutz",
    "part": "Teil1",
    "dateFrom": "2024-01-01",
    "limit": 10
  }'
```

### Request Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `query` | string | Full-text search | `"Finanzmarktaufsicht"` |
| `documentType` | string | Document type | `"Verordnung"`, `"Bundesgesetz"` |
| `part` | string | BGBl part | `"Teil1"` (I), `"Teil2"` (II), `"Teil3"` (III) |
| `gazetteNumber` | string | Exact BGBl number | `"BGBl. II Nr. 352/2019"` |
| `dateFrom` | date | Publication date from (YYYY-MM-DD) | `"2024-01-01"` |
| `dateTo` | date | Publication date to (YYYY-MM-DD) | `"2025-12-31"` |
| `offset` | int | Pagination offset (default 0) | `0` |
| `limit` | int | Results per page (max 100, default 20) | `10` |

### Query Parameters

| Param | Values | Description |
|-------|--------|-------------|
| `sort` | `relevance`, `title`, `date` | Sort field (default: relevance) |
| `order` | `asc`, `desc` | Sort direction |

## Response Fields

| Field | Description |
|-------|-------------|
| `docId` | Display ID (e.g., ATBR1234) — internal only |
| `title` | Full title of the legislation |
| `shortTitle` | Short title |
| `gazetteNumber` | BGBl number (e.g., "BGBl. II Nr. 352/2019") |
| `part` | BGBl part (Teil1/Teil2/Teil3) |
| `publicationDate` | Publication date (YYYY-MM-DD) |
| `documentType` | Bundesgesetz, Verordnung, etc. |
| `authority` | Issuing authority |
| `eli` | European Legislation Identifier |
| `celexNumbers` | EU CELEX directive references |

## Examples

### Search Recent Legislation

```bash
cdx-at -s -X POST "cdx-at://search/ATBR" \
  -H 'Content-Type: application/json' \
  -d '{"query": "Klimaschutz", "dateFrom": "2024-01-01", "limit": 5}' \
  | jq '.results[] | {docId, title, gazetteNumber, publicationDate}'
```

### Search by BGBl Number

```bash
cdx-at -s -X POST "cdx-at://search/ATBR" \
  -H 'Content-Type: application/json' \
  -d '{"gazetteNumber": "BGBl. I Nr. 58/2018", "limit": 1}' \
  | jq '.results[0]'
```
