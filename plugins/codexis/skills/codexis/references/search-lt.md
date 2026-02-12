# Legal Literature Search (LT)

Search for legal publications and articles.

## cdx Usage
Use `cdx` for requests. It accepts standard curl flags and `cdx://` URLs.

```bash
cdx -s -X POST "cdx://search/LT" \
  -H 'Content-Type: application/json' \
  -d '{"query": "smlouva", "limit": 5}'
```

## Endpoint

```
POST cdx://search/LT
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
  "issuedTo": "date"
}
```

## Response Schema

```json
{
  "results": [
    {
      "docId": "LT146061",
      "docUrl": "cdx://doc/LT146061/text",
      "docType": "Životní situace",
      "title": "Životní situace - Uzavírání partnerství podle občanského zákoníku",
      "snippet": "text with <mark>highlights</mark>",
      "source": "ATLAS consulting spol. s r.o.",
      "createdDate": "2025-01-01"
    }
  ],
  "totalResults": 5678,
  "offset": 0,
  "limit": 10
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Document ID for retrieval |
| `docUrl` | Literature text URL |
| `docType` | Literature content type |
| `source` | Publisher/source |
| `createdDate` | Publication/update date |

## Examples

### Search Legal Articles

```bash
cdx -s -X POST "cdx://search/LT" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "odpovědnost za škodu",
    "limit": 10
  }' | jq '.results[] | {docId, docType, title, source}'
```

### Search Recent Publications

```bash
cdx -s -X POST "cdx://search/LT" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "GDPR ochrana údajů",
    "issuedFrom": "2024-01-01",
    "sort": "DATE",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .createdDate}'
```

### Search by Topic

```bash
cdx -s -X POST "cdx://search/LT" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "insolvence úpadek",
    "limit": 15
  }' | jq '.results[] | {docId, docType, title}'
```

## Working with Literature

### Get Article Text

LT documents often expose a generated TOC (`SEKCE...` element IDs). Prefer TOC + marker extraction when available; otherwise use full text:

```bash
DOC_ID="LT146061"
cdx -s "cdx://doc/${DOC_ID}/toc" | jq '.'
cdx -s "cdx://doc/${DOC_ID}/text"
```

### Find Related Legislation

```bash
cdx -s "cdx://doc/LT146061/related?type=SOUVISEJICI_LEGISLATIVA_CR" | \
  jq '.results[] | {docId, title}'
```

### Find Related Case Law

```bash
cdx -s "cdx://doc/LT146061/related?type=SOUVISEJICI_JUDIKATURA" | \
  jq '.results[] | {docId, title}'
```

## Research Workflow

1. Search literature for academic perspective:
```bash
cdx -s -X POST "cdx://search/LT" \
  -H 'Content-Type: application/json' \
  -d '{"query": "bezdůvodné obohacení", "limit": 5}'
```

2. Find related legislation cited in articles:
```bash
cdx -s "cdx://doc/LT_DOC_ID/related?type=SOUVISEJICI_LEGISLATIVA_CR"
```

3. Find supporting case law:
```bash
cdx -s "cdx://doc/LT_DOC_ID/related?type=SOUVISEJICI_JUDIKATURA"
```
