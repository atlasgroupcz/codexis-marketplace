# Slovak Legislation Search (SK)

Search for Slovak laws and regulations.

## cdx Usage
Use `cdx` for requests. It is opinionated: it runs silently by default, and `-d` implies `POST` plus `Content-Type: application/json` unless you override them.

```bash
cdx "cdx://search/SK" \
  -d '{"query": "daň", "limit": 5}'
```

## Endpoint

```
POST cdx://search/SK
JSON request body
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

Runtime note: on current backend, `/text` and `/toc` for SK require a **version ID** (`SK..._YYYY_MM_DD`), not a base ID.

```json
{
  "results": [
    {
      "docId": "SK48_2026_01_01",
      "docUrl": "cdx://doc/SK48_2026_01_01/text",
      "docNumber": "40/1964 Zb.",
      "docType": "Zákon",
      "source": "www.slov-lex.sk",
      "amount": "19/1964",
      "vyhlaseno": "1964-03-05",
      "schvaleno": "1964-02-26",
      "author": "Národné zhromaždenie Československej socialistickej republiky",
      "snippet": "text with <mark>highlights</mark>",
      "title": "40/1964 Zb. <mark>Občiansky</mark> <mark>zákonník</mark>"
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
| `docId` | Version-specific ID (required for `/text` and `/toc`) |
| `docType` | Document type |
| `schvaleno` | Approval date |
| `vyhlaseno` | Published date |
| `docUrl` | Link to document text endpoint |

## Document Types (typ facet)

- `Zákon` - Law
- `Vyhláška` - Decree
- `Oznámenie` - Notice
- `Opatrenie` - Measure
- `Nariadenie vlády` - Government regulation

## Examples

### Search Slovak Laws

```bash
cdx "cdx://search/SK" \
  -d '{
    "query": "občiansky zákonník",
    "typ": ["Zákon"],
    "limit": 10
  }' | jq '.results[] | {docId, title}'
```

### Search Recent Slovak Legislation

```bash
cdx "cdx://search/SK" \
  -d '{
    "query": "daň z príjmov",
    "issuedFrom": "2024-01-01",
    "sort": "DATE",
    "limit": 10
  }' | jq '.results[] | {docId, title, approved: .schvaleno, published: .vyhlaseno}'
```

### Compare with Czech Legislation

Slovak and Czech laws often have similar structure. Search both:

```bash
# Czech
cdx "cdx://search/CR" \
  -d '{"query": "zákoník práce", "validNow": true, "limit": 3}' | \
  jq '.results[] | {source: "CR", docId, title}'

# Slovak
cdx "cdx://search/SK" \
  -d '{"query": "zákonník práce", "limit": 3}' | \
  jq '.results[] | {source: "SK", docId, title}'
```

## Working with Slovak Documents

Slovak documents have TOC and versions like Czech legislation:

```bash
DOC_ID="SK48_2026_01_01"

# Get full text
cdx "cdx://doc/${DOC_ID}/text"

# Get TOC
cdx "cdx://doc/${DOC_ID}/toc" | jq '.'

# Get versions
cdx "cdx://doc/${DOC_ID}/versions" | jq '.'
```

See `czech-legislature.md` for detailed text extraction techniques - same patterns apply.
