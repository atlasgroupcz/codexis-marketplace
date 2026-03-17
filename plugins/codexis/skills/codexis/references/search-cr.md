# Czech Legislation Search (CR)

Search for Czech laws, decrees, regulations, and municipal documents.

## cdx Usage
Use `cdx` for requests. It is opinionated: it runs silently by default, and `-d` implies `POST` plus `Content-Type: application/json` unless you override them.

```bash
cdx "cdx://search/CR" \
  -d '{"query": "občanský zákoník", "limit": 5}'
```

## Endpoint

```
POST cdx://search/CR
JSON request body
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

Runtime note: on current backend, `/text` and `/toc` for CR require a **version ID** (`CR..._YYYY_MM_DD`), not a base ID.

```json
{
  "results": [
    {
      "docId": "CR26785_2026_01_01",
      "docUrl": "cdx://doc/CR26785_2026_01_01/text",
      "docNumber": "89/2012",
      "docType": "Zákon",
      "source": "Sbírka zákonů",
      "castka": "33/2012",
      "schvaleno": "2012-02-03",
      "platnyod": "2012-03-22",
      "ucinnyod": "2014-01-01",
      "zruseno": null,
      "note": null,
      "snippet": "matched text with <mark>highlights</mark>",
      "title": "89/2012 Sb. Zákon <mark>občanský</mark> <mark>zákoník</mark>"
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
| `docId` | Version-specific ID (required for `/text` and `/toc`) |
| `docType` | Document type (Zákon, Vyhláška, etc.) |
| `schvaleno` | Approval date |
| `platnyod` | Valid from (published in collection) |
| `ucinnyod` | Effective from (legally binding) |
| `zruseno` | Repealed date (null = still valid) |
| `docUrl` | Link to document text endpoint |

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
cdx "cdx://search/CR" \
  -d '{
    "query": "daň z příjmů",
    "validNow": true,
    "typ": ["Zákon"],
    "limit": 10
  }' | jq '.results[] | {docId, title, effective: .ucinnyod}'
```

### Search Laws Valid at Specific Date

```bash
cdx "cdx://search/CR" \
  -d '{
    "query": "občanský zákoník",
    "validAt": "2010-01-01",
    "limit": 5
  }' | jq '.results[] | {docId, title}'
```

### Search Recent Changes

```bash
cdx "cdx://search/CR" \
  -d '{
    "query": "stavební zákon",
    "changedFrom": "2024-01-01",
    "sortBy": "DATE",
    "limit": 10
  }' | jq '.results[] | {docId, title, approved: .schvaleno, effective: .ucinnyod}'
```

### Search Municipal Documents

```bash
cdx "cdx://search/CR" \
  -d '{
    "query": "Praha",
    "typ": ["Obecně závazná vyhláška", "Nařízení obce"],
    "validNow": true,
    "limit": 10
  }' | jq '.results[] | {docId, title, type: .docType}'
```

### Get All Document Types

```bash
# Search broadly and extract unique types
cdx "cdx://search/CR" \
  -d '{"query": "*", "limit": 50}' \
  | jq '[.results[].docType] | unique'
```

## Processing Results

### Extract Document IDs for Further Processing

```bash
cdx "cdx://search/CR" \
  -d '{"query": "zákoník práce", "validNow": true, "limit": 5}' \
  | jq -r '.results[].docId'
```

### Get Version-Specific IDs

`docId` is already version-specific in CR search results.

```bash
cdx "cdx://search/CR" \
  -d '{"query": "trestní zákoník", "validNow": true, "limit": 3}' \
  | jq -r '.results[].docId'
```

### Filter by Multiple Document Types

```bash
cdx "cdx://search/CR" \
  -d '{
    "query": "ochrana osobních údajů",
    "typ": ["Zákon", "Nařízení vlády", "Vyhláška"],
    "validNow": true,
    "limit": 20
  }' | jq '.results | group_by(.docType) | map({type: .[0].docType, count: length})'
```
