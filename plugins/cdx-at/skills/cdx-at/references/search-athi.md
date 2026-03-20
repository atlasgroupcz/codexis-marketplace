# Austrian Consolidated Law History Search (ATHI — History)

Consolidated federal law norms (Bundesnormen) with amendment chains. Each record represents a single consolidated norm provision with its full amendment history linking back to BGBl publications.

## Search Request

```bash
cdx-at -s -X POST "cdx-at://search/ATHI" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "Sozialversicherung",
    "abbreviation": "ASVG",
    "limit": 10
  }'
```

### Request Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `query` | string | Full-text search | `"Sozialversicherung"` |
| `documentType` | string | Document type code | `"BG"` (Bundesgesetz), `"V"` (Verordnung) |
| `abbreviation` | string | Law abbreviation | `"ASVG"`, `"StGB"`, `"B-VG"`, `"ABGB"` |
| `dateFrom` | date | Effective date from (YYYY-MM-DD) | `"2024-01-01"` |
| `dateTo` | date | Effective date to (YYYY-MM-DD) | `"2025-12-31"` |
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
| `docId` | Display ID (e.g., ATHI1234) — internal only |
| `shortTitle` | Short title of the law |
| `title` | Full title |
| `abbreviation` | Law abbreviation (e.g., StGB, ASVG) |
| `articleParagraph` | Article/paragraph reference (e.g., "§ 165") |
| `lawNumber` | Gesetzesnummer (internal RIS law number) |
| `effectiveDate` | Effective date (YYYY-MM-DD) |
| `expiryDate` | Expiry date or null if still in force |
| `publicationOrgan` | Original publication (e.g., "BGBl. Nr. 189/1955") |
| `documentType` | BG, V, etc. |
| `amendments` | Array of BGBl references that amended this norm |
| `eli` | European Legislation Identifier |

## Examples

### Search by Law Abbreviation

```bash
cdx-at -s -X POST "cdx-at://search/ATHI" \
  -H 'Content-Type: application/json' \
  -d '{"abbreviation": "StGB", "limit": 10}' \
  | jq '.results[] | {docId, shortTitle, articleParagraph, effectiveDate}'
```

### Search Currently Effective Norms

```bash
cdx-at -s -X POST "cdx-at://search/ATHI" \
  -H 'Content-Type: application/json' \
  -d '{"query": "Datenschutz", "dateFrom": "2026-01-01", "limit": 5}' \
  | jq '.results[] | {docId, abbreviation, shortTitle, amendments}'
```

### Find Norms by Amendment

```bash
# Search for norms amended by a specific BGBl
cdx-at -s -X POST "cdx-at://search/ATHI" \
  -H 'Content-Type: application/json' \
  -d '{"query": "BGBl. I Nr. 58/2018", "limit": 10}' \
  | jq '.results[] | {docId, abbreviation, articleParagraph, amendments}'
```
