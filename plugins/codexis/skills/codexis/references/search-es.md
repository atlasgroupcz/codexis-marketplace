# EU Court Decisions Search (ES)

Search for EU Court of Justice and ECHR rulings.

## cdx Usage
Use `cdx` for requests. It accepts standard curl flags and `cdx://` URLs.

```bash
cdx -s -X POST "cdx://search/ES" \
  -H 'Content-Type: application/json' \
  -d '{"query": "privacy", "limit": 5}'
```

## Endpoint

```
POST cdx://search/ES
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
      "docId": "ES64859",
      "docUrl": "cdx://doc/ES64859/text",
      "title": "Věc T-110/23: Rozsudek Tribunálu...",
      "snippet": "text with <mark>highlights</mark>",
      "docType": "Rozsudek",
      "createdDate": "2024-11-13",
      "celex": "62023TJ0110",
      "ecli": "ECLI:EU:T:2024:805",
      "court": "Tribunál",
      "city": "Luxembourg",
      "subject": null
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
| `docUrl` | Decision text URL |
| `celex` | CELEX number |
| `ecli` | European Case Law Identifier |
| `docType` | Decision type |

## Decision Types (typ facet)

- `Rozsudek` - Judgment
- `Stanovisko` - Opinion
- `Usnesení` - Order

## Examples

### Search EU Court Judgments

```bash
cdx -s -X POST "cdx://search/ES" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "ochrana spotřebitele",
    "typ": ["Rozsudek"],
    "limit": 10
  }' | jq '.results[] | {docId, title, celex}'
```

### Search GDPR-Related Decisions

```bash
cdx -s -X POST "cdx://search/ES" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "GDPR osobní údaje",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .createdDate}'
```

### Search Recent Decisions

```bash
cdx -s -X POST "cdx://search/ES" \
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
cdx -s -X POST "cdx://search/ES" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "C-383/23",
    "limit": 5
  }' | jq '.results[] | {docId, title, celex}'
```

## Working with EU Court Decisions

### Get Decision Text

ES documents often expose a lightweight generated TOC (`SEKCE...` element IDs). Prefer TOC + marker extraction when available; otherwise use full text:

```bash
DOC_ID="ES64859"
cdx -s "cdx://doc/${DOC_ID}/toc" | jq '.'
cdx -s "cdx://doc/${DOC_ID}/text"
```

### Find Related EU Legislation

```bash
cdx -s "cdx://doc/ES64859/related?type=SOUVISEJICI_PREDPISY_EU" | \
  jq '.results[] | {docId, title}'
```

### Find Related Czech Legislation

```bash
cdx -s "cdx://doc/ES64859/related?type=SOUVISEJICI_LEGISLATIVA_CR" | \
  jq '.results[] | {docId, title}'
```

## CELEX Format for Court Decisions

EU Court CELEX numbers:
- `6` = Court of Justice case law
- `2020` = Year
- `CJ` = Court of Justice judgment
- `0123` = Case number

Example: `62020CJ0123` = Judgment in Case C-123/20
