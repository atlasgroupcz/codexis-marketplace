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

  "relatedWithItem": "string - related CR docId",
  "relatedWithItemPart": "string - specific paragraph (use with relatedWithItem)"
}
```

## Response Schema

```json
{
  "results": [
    {
      "commentId": "COMMENT112807",
      "title": "Nový občanský zákoník. Smluvní právo - Kapitola 9 Smlouvy se spotřebiteli",
      "snippet": "text with <mark>highlights</mark>",
      "nameSnippet": "title with <mark>highlights</mark>",
      "partName": "Kapitola 9 Smlouvy se spotřebiteli",
      "bookId": "BOOKS1000070",
      "editionId": "BOOKS1000070_2017_09_22",
      "validFromDate": null,
      "tags": ["LIBERIS", "MONITOR_CIRKEV", "MONITOR_OBECNI_SAMOSPRAVA"]
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
| `commentId` | Commentary ID for retrieval |
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
  }' | jq '.results[] | {id: .commentId, title, part: .partName}'
```

### Find Commentaries for Specific Law

```bash
cdx -s -X POST "cdx://search/COMMENT" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "občanský zákoník",
    "relatedWithItem": "CR26785",
    "limit": 10
  }' | jq '.results[] | {id: .commentId, title, part: .partName}'
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
  }' | jq '.results[] | {id: .commentId, title, part: .partName}'
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
  }' | jq '.results[] | {id: .commentId, title}'
```

## Working with Commentaries

### Get Commentary Text

Commentaries do not have TOC - fetch full text:

```bash
COMMENT_ID="COMMENT112807"
cdx -s "cdx://doc/${COMMENT_ID}/text"
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
  -d '{"query": "nájem bytu", "relatedWithItem": "CR26785", "limit": 5}'
```

3. Get commentary text:
```bash
cdx -s "cdx://doc/COMMENT_ID/text"
```
