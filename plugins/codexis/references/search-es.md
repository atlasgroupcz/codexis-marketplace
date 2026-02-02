# EU Court Decisions Search (ES)

Search for EU Court of Justice and ECHR rulings.

## Endpoint

```
POST ${CODEXIS_API_URL}/rest/cdx-api/search/ES
Content-Type: application/json
```

## Request Schema

```json
{
  "query": "string (required) - fulltext search",
  "limit": "integer (1-50, default: 10)",
  "offset": "integer (default: 0)",
  "sort": "RELEVANCE | DATE | NAME",
  "sortOrder": "ASC | DESC (default: DESC)",

  "typ": ["Rozsudek", "Stanovisko", ...],

  "issuedFrom": "date (YYYY-MM-DD)",
  "issuedTo": "date"
}
```

## Response Schema

```json
{
  "results": [
    {
      "docId": "ES12345",
      "title": "C-123/20: Rozsudek Soudního dvora...",
      "snippet": "text with <mark>highlights</mark>",
      "nameSnippet": "title with <mark>highlights</mark>",
      "docType": "Rozsudek",
      "createdDate": "2024-01-15",
      "celex": "62020CJ0123"
    }
  ],
  "totalResults": 1234,
  "offset": 0,
  "limit": 10
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Document ID for retrieval |
| `celex` | CELEX number |
| `docType` | Decision type |

## Decision Types (typ facet)

- `Rozsudek` - Judgment
- `Stanovisko` - Opinion
- `Usnesení` - Order

## Examples

### Search EU Court Judgments

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/ES" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "ochrana spotřebitele",
    "typ": ["Rozsudek"],
    "limit": 10
  }' | jq '.results[] | {docId, title, celex}'
```

### Search GDPR-Related Decisions

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/ES" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "GDPR osobní údaje",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .createdDate}'
```

### Search Recent Decisions

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/ES" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "volný pohyb zboží",
    "issuedFrom": "2024-01-01",
    "sort": "DATE",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .createdDate}'
```

### Search by Case Number

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/ES" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "C-383/23",
    "limit": 5
  }' | jq '.results[] | {docId, title, celex}'
```

## Working with EU Court Decisions

### Get Decision Text

ES documents do not have TOC - fetch full text:

```bash
DOC_ID="ES12345"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/text"
```

### Find Related EU Legislation

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/ES12345/related?type=SOUVISEJICI_PREDPISY_EU" | \
  jq '.results[] | {docId, title}'
```

### Find Related Czech Legislation

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/ES12345/related?type=SOUVISEJICI_LEGISLATIVA_CR" | \
  jq '.results[] | {docId, title}'
```

## CELEX Format for Court Decisions

EU Court CELEX numbers:
- `6` = Court of Justice case law
- `2020` = Year
- `CJ` = Court of Justice judgment
- `0123` = Case number

Example: `62020CJ0123` = Judgment in Case C-123/20
