# Slovak Legislation Search (SK)

Search for Slovak laws and regulations.

## Endpoint

```
POST ${CODEXIS_API_URL}/rest/cdx-api/search/SK
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

  "typ": ["Zákon", "Vyhláška", "Oznámenie", "Opatrenie", ...],

  "issuedFrom": "date (YYYY-MM-DD)",
  "issuedTo": "date",
  "effectiveFrom": "date",
  "effectiveTo": "date"
}
```

## Response Schema

```json
{
  "results": [
    {
      "main": {
        "docId": "SK12345",
        "title": "40/1964 Zb. Občiansky zákonník",
        "docType": "Zákon",
        "docNumber": "40/1964",
        "source": "Zbierka zákonov",
        "authors": ["Národná rada"],
        "schvaleno": "1964-02-26",
        "platnyOd": "1964-03-05",
        "ucinnyOd": "1964-04-01",
        "zruseno": null,
        "note": null
      },
      "timecut": {
        "timecutId": "SK12345_2026_01_01",
        "docId": "SK12345",
        "validFrom": "2026-01-01",
        "validTo": null,
        "derogation": false
      },
      "snippet": "text with <mark>highlights</mark>",
      "nameSnippet": "title with <mark>highlights</mark>"
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
| `docId` | Base document ID |
| `timecutId` | Version-specific ID |
| `docType` | Document type |
| `schvaleno` | Approval date |
| `platnyOd` | Valid from (published) |
| `ucinnyOd` | Effective from |
| `zruseno` | Repealed date (null = valid) |

## Document Types (typ facet)

- `Zákon` - Law
- `Vyhláška` - Decree
- `Oznámenie` - Notice
- `Opatrenie` - Measure
- `Nariadenie vlády` - Government regulation

## Examples

### Search Slovak Laws

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/SK" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "občiansky zákonník",
    "typ": ["Zákon"],
    "limit": 10
  }' | jq '.results[] | {docId: .main.docId, title: .main.title}'
```

### Search Recent Slovak Legislation

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/SK" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "daň z príjmov",
    "issuedFrom": "2024-01-01",
    "sort": "DATE",
    "limit": 10
  }' | jq '.results[] | {docId: .main.docId, title: .main.title, date: .main.ucinnyOd}'
```

### Compare with Czech Legislation

Slovak and Czech laws often have similar structure. Search both:

```bash
# Czech
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/CR" \
  -H 'Content-Type: application/json' \
  -d '{"query": "zákoník práce", "validNow": true, "limit": 3}' | \
  jq '.results[] | {source: "CR", docId: .main.docId, title: .main.title}'

# Slovak
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/SK" \
  -H 'Content-Type: application/json' \
  -d '{"query": "zákonník práce", "limit": 3}' | \
  jq '.results[] | {source: "SK", docId: .main.docId, title: .main.title}'
```

## Working with Slovak Documents

Slovak documents have TOC and versions like Czech legislation:

```bash
DOC_ID="SK12345"

# Get full text
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/text"

# Get TOC
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/toc" | jq '.'

# Get versions
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/versions" | jq '.'
```

See `czech-legislature.md` for detailed text extraction techniques - same patterns apply.
