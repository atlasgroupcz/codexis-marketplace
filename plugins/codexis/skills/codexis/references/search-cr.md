# Czech Legislation Search (CR)

Search for Czech laws, decrees, regulations, and municipal documents.

## Endpoint

```
POST ${CODEXIS_API_URL}/rest/cdx-api/search/CR
Content-Type: application/json
```

## Request Schema

```json
{
  "query": "string (required) - fulltext search",
  "limit": "integer (1-50, default: 10)",
  "offset": "integer (default: 0)",
  "sortBy": "RELEVANCE | DATE | NAME",
  "sortOrder": "ASC | DESC (default: DESC)",

  "typ": ["Zákon", "Vyhláška", "Nařízení vlády", ...],
  "autor": ["Ministerstvo financí", "Parlament", ...],

  "validNow": "boolean - only currently valid",
  "validAt": "date (YYYY-MM-DD) - valid at specific date",

  "issuedFrom": "date",
  "issuedTo": "date",
  "effectiveFrom": "date",
  "effectiveTo": "date",
  "approvedFrom": "date",
  "approvedTo": "date",
  "changedFrom": "date",
  "changedTo": "date"
}
```

**Note:** `validNow` and `validAt` are mutually exclusive.

## Response Schema

```json
{
  "results": [
    {
      "main": {
        "docId": "CR26785",
        "title": "89/2012 Sb. Zákon občanský zákoník",
        "docType": "Zákon",
        "docNumber": "89/2012",
        "source": "Sbírka zákonů",
        "castka": "33/2012",
        "authors": ["Parlament"],
        "schvaleno": "2012-02-03",
        "platnyOd": "2012-03-22",
        "ucinnyOd": "2014-01-01",
        "zruseno": null,
        "note": null
      },
      "timecut": {
        "timecutId": "CR26785_2026_01_01",
        "docId": "CR26785",
        "validFrom": "2026-01-01",
        "validTo": null,
        "derogation": false
      },
      "snippet": "matched text with <mark>highlights</mark>",
      "nameSnippet": "title with <mark>highlights</mark>"
    }
  ],
  "totalResults": 5046,
  "offset": 0,
  "limit": 5
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Base document ID (use for /text, /toc, /versions) |
| `timecutId` | Version-specific ID (use for version-specific text) |
| `docType` | Document type (Zákon, Vyhláška, etc.) |
| `schvaleno` | Approval date |
| `platnyOd` | Valid from (published in collection) |
| `ucinnyOd` | Effective from (legally binding) |
| `zruseno` | Repealed date (null = still valid) |
| `derogation` | True if this version was derogated |

## Document Types (typ facet)

Common values:
- `Zákon` - Law
- `Vyhláška` - Decree
- `Nařízení vlády` - Government regulation
- `Sdělení` - Communication
- `Obecně závazná vyhláška` - Municipal ordinance
- `Nařízení obce` - Municipal regulation

## Examples

### Search for Currently Valid Laws

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/CR" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "daň z příjmů",
    "validNow": true,
    "typ": ["Zákon"],
    "limit": 10
  }' | jq '.results[] | {docId: .main.docId, title: .main.title, effective: .main.ucinnyOd}'
```

### Search Laws Valid at Specific Date

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/CR" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "občanský zákoník",
    "validAt": "2010-01-01",
    "limit": 5
  }' | jq '.results[] | {docId: .main.docId, title: .main.title}'
```

### Search Recent Changes

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/CR" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "stavební zákon",
    "changedFrom": "2024-01-01",
    "sortBy": "DATE",
    "limit": 10
  }' | jq '.results[] | {docId: .main.docId, title: .main.title, changed: .timecut.validFrom}'
```

### Search Municipal Documents

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/CR" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "Praha",
    "typ": ["Obecně závazná vyhláška", "Nařízení obce"],
    "validNow": true,
    "limit": 10
  }' | jq '.results[] | {docId: .main.docId, title: .main.title, type: .main.docType}'
```

### Get All Document Types

```bash
# Search broadly and extract unique types
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/CR" \
  -H 'Content-Type: application/json' \
  -d '{"query": "*", "limit": 50}' \
  | jq '[.results[].main.docType] | unique'
```

## Processing Results

### Extract Document IDs for Further Processing

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/CR" \
  -H 'Content-Type: application/json' \
  -d '{"query": "zákoník práce", "validNow": true, "limit": 5}' \
  | jq -r '.results[].main.docId'
```

### Get Version-Specific IDs

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/CR" \
  -H 'Content-Type: application/json' \
  -d '{"query": "trestní zákoník", "validNow": true, "limit": 3}' \
  | jq -r '.results[].timecut.timecutId'
```

### Filter by Multiple Document Types

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/CR" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "ochrana osobních údajů",
    "typ": ["Zákon", "Nařízení vlády", "Vyhláška"],
    "validNow": true,
    "limit": 20
  }' | jq '.results | group_by(.main.docType) | map({type: .[0].main.docType, count: length})'
```
