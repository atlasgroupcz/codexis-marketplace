# Legal Commentaries Search (COMMENT)

Search for LIBERIS legal commentaries on Czech legislation.

## cdx Usage
Use `cdx` for requests. It accepts standard curl flags and `cdx://` URLs.

```bash
cdx -s -X POST "cdx://search/COMMENT" \
  -H 'Content-Type: application/json' \
  -d '{"query": "nájem bytu", "limit": 5}'
```

## Endpoint

```
POST cdx://search/COMMENT
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

  "issuedFrom": "date (YYYY-MM-DD)",
  "issuedTo": "date",

  "relatedWithItem": "string - related CR docId (base or version ID)",
  "relatedWithItemPart": "string - specific paragraph (use with relatedWithItem)"
}
```

## Response Schema

```json
{
  "results": [
    {
      "docId": "COMMENT76521",
      "docUrl": "cdx://doc/COMMENT76521/text",
      "title": "Listina základních práv a svobod - judikatorní komentář - Čl. 11 [Vlastnické právo]",
      "snippet": "text with <mark>highlights</mark>",
      "partName": "Čl. 11 [Vlastnické právo]",
      "bookId": "BOOKS1000128",
      "editionId": "BOOKS1000128_2017_03_01",
      "validFromDate": "2017-03-01",
      "tags": ["LIBERIS"]
    }
  ],
  "totalResults": 17310,
  "offset": 0,
  "limit": 3
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Commentary ID for retrieval |
| `docUrl` | Commentary text URL |
| `partName` | Chapter/section name |
| `bookId` | Parent book ID |
| `editionId` | Specific edition |
| `tags` | Content categories |

## Examples

### Search Commentaries on a Topic

```bash
cdx -s -X POST "cdx://search/COMMENT" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "kupní smlouva",
    "limit": 10
  }' | jq '.results[] | {docId, title, part: .partName}'
```

### Find Commentaries for Specific Law

```bash
cdx -s -X POST "cdx://search/COMMENT" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "občanský zákoník",
    "relatedWithItem": "CR26785",
    "limit": 10
  }' | jq '.results[] | {docId, title, part: .partName}'
```

### Find Commentary for Specific Paragraph

```bash
cdx -s -X POST "cdx://search/COMMENT" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "*",
    "relatedWithItem": "CR26785",
    "relatedWithItemPart": "paragraf89",
    "limit": 10
  }' | jq '.results[] | {docId, title, part: .partName}'
```

### Search Recent Commentaries

```bash
cdx -s -X POST "cdx://search/COMMENT" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "GDPR ochrana údajů",
    "issuedFrom": "2023-01-01",
    "sort": "DATE",
    "limit": 10
  }' | jq '.results[] | {docId, title}'
```

## Working with Commentaries

### Get Commentary Text

Commentaries do not have TOC (`/toc` currently returns HTTP 500) - fetch full text:

```bash
DOC_ID="COMMENT76521"
cdx -s "cdx://doc/${DOC_ID}/text"
```

### Find Related Legislation

```bash
cdx -s "cdx://doc/COMMENT112807/related?type=SOUVISEJICI_LEGISLATIVA_CR" | \
  jq '.results[] | {docId, title}'
```

## Workflow: Research a Legal Topic

1. Search for relevant law:
```bash
cdx -s -X POST "cdx://search/CR" \
  -H 'Content-Type: application/json' \
  -d '{"query": "nájem bytu", "validNow": true, "limit": 3}'
```

2. Find commentaries for that law:
```bash
cdx -s -X POST "cdx://search/COMMENT" \
  -H 'Content-Type: application/json' \
  -d '{"query": "nájem bytu", "relatedWithItem": "CR26785_2026_01_01", "limit": 5}'
```

3. Get commentary text:
```bash
cdx -s "cdx://doc/COMMENT_DOC_ID/text"
```
