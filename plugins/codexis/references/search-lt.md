# Legal Literature Search (LT)

Search for legal publications and articles.

## Endpoint

```
POST ${CODEXIS_API_URL}/rest/cdx-api/search/LT
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
      "docId": "LT12345",
      "title": "Odpovědnost za škodu v občanském právu",
      "snippet": "text with <mark>highlights</mark>",
      "nameSnippet": "title with <mark>highlights</mark>",
      "author": "Jan Novák",
      "source": "Právní rozhledy",
      "createdDate": "2024-03-15"
    }
  ],
  "totalResults": 5678,
  "offset": 0,
  "limit": 10
}
```

## Examples

### Search Legal Articles

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/LT" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "odpovědnost za škodu",
    "limit": 10
  }' | jq '.results[] | {docId, title, author, source}'
```

### Search Recent Publications

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/LT" \
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
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/LT" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "insolvence úpadek",
    "limit": 15
  }' | jq '.results[] | {docId, title, author}'
```

## Working with Literature

### Get Article Text

Literature documents do not have TOC - fetch full text:

```bash
DOC_ID="LT12345"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/text"
```

### Find Related Legislation

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/LT12345/related?type=SOUVISEJICI_LEGISLATIVA_CR" | \
  jq '.results[] | {docId, title}'
```

### Find Related Case Law

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/LT12345/related?type=SOUVISEJICI_JUDIKATURA" | \
  jq '.results[] | {docId, title}'
```

## Research Workflow

1. Search literature for academic perspective:
```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/LT" \
  -H 'Content-Type: application/json' \
  -d '{"query": "bezdůvodné obohacení", "limit": 5}'
```

2. Find related legislation cited in articles:
```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/LT_DOC_ID/related?type=SOUVISEJICI_LEGISLATIVA_CR"
```

3. Find supporting case law:
```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/LT_DOC_ID/related?type=SOUVISEJICI_JUDIKATURA"
```
