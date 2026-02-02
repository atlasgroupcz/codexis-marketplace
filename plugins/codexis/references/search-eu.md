# EU Legislation Search (EU)

Search for EU regulations, directives, and decisions.

## Endpoint

```
POST ${CODEXIS_API_URL}/rest/cdx-api/search/EU
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

  "typ": ["Nařízení", "Směrnice", "Rozhodnutí", ...],
  "zdroj": ["Úřední věstník Evropské unie"],
  "zdrojUveu": ["L", "C"],
  "autor": ["Evropský parlament", "Evropská komise", ...],
  "oblast": ["Průmyslová politika a vnitřní trh", "Zemědělství", ...],

  "issuedFrom": "date (YYYY-MM-DD)",
  "issuedTo": "date",
  "approvedFrom": "date",
  "approvedTo": "date",
  "effectiveFrom": "date",
  "effectiveTo": "date"
}
```

## Response Schema

```json
{
  "results": [
    {
      "docId": "EU213382",
      "title": "2023/1795: Prováděcí rozhodnutí Komise (EU) 2023/1795...",
      "snippet": "text with <mark>highlights</mark>",
      "nameSnippet": "title with <mark>highlights</mark>",
      "docType": "Rozhodnutí",
      "source": "Úřední věstník Evropské unie",
      "createdDate": "2023-09-20",
      "sourceUveu": "L",
      "language": "CZ",
      "domain": "Průmyslová politika a vnitřní trh",
      "celex": "32023D1795",
      "validFrom": "2023-09-20",
      "validTo": null,
      "approvalDate": "2023-07-10",
      "docNumber": "2023/1795"
    }
  ],
  "totalResults": 384,
  "offset": 0,
  "limit": 3
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Document ID for retrieval |
| `celex` | CELEX number (EU document identifier) |
| `docType` | Document type (Nařízení, Směrnice, etc.) |
| `sourceUveu` | Official Journal series (L or C) |
| `domain` | Legal domain/area |
| `validFrom` | Entry into force date |
| `validTo` | Expiry date (null = still valid) |

## Document Types (typ facet)

- `Nařízení` - Regulation (directly applicable)
- `Směrnice` - Directive (requires transposition)
- `Rozhodnutí` - Decision
- `Doporučení` - Recommendation
- `Stanovisko` - Opinion
- `Soudní informace` - Court information

## Official Journal Series (zdrojUveu facet)

- `L` - Legislation (binding acts)
- `C` - Information and Notices

## Examples

### Search GDPR-Related Documents

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/EU" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "GDPR ochrana osobních údajů",
    "limit": 10
  }' | jq '.results[] | {docId, title, celex, docType}'
```

### Search EU Regulations Only

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/EU" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "potraviny bezpečnost",
    "typ": ["Nařízení"],
    "limit": 10
  }' | jq '.results[] | {docId, title, celex}'
```

### Search by CELEX Number

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/EU" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "32016R0679",
    "limit": 5
  }' | jq '.results[] | {docId, celex, title}'
```

### Search Recent EU Legislation

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/EU" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "umělá inteligence AI",
    "issuedFrom": "2024-01-01",
    "sort": "DATE",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .createdDate}'
```

### Search by Domain

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/EU" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "*",
    "oblast": ["Životní prostředí"],
    "typ": ["Směrnice"],
    "limit": 10
  }' | jq '.results[] | {docId, title, domain}'
```

### Search Directives for Transposition

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/EU" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "digitální služby",
    "typ": ["Směrnice"],
    "effectiveFrom": "2023-01-01",
    "limit": 10
  }' | jq '.results[] | {docId, title, effective: .validFrom}'
```

## Working with EU Documents

### Get Document Text

EU documents have TOC and versions:

```bash
DOC_ID="EU213382"

# Get full text
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/text"

# Get TOC
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/toc" | jq '.'

# Get versions
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/versions" | jq '.'
```

### Find Implementing Czech Legislation

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/EU213382/related?type=SOUVISEJICI_LEGISLATIVA_CR&limit=10" | \
  jq '.results[] | {docId, title}'
```

### Group by Document Type

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/EU" \
  -H 'Content-Type: application/json' \
  -d '{"query": "finanční trhy", "limit": 50}' | \
  jq '.results | group_by(.docType) | map({type: .[0].docType, count: length})'
```

## CELEX Number Format

CELEX numbers follow a pattern:
- `3` = Legislation
- `2023` = Year
- `R` = Regulation, `L` = Directive, `D` = Decision
- `0679` = Sequential number

Example: `32016R0679` = GDPR (Regulation 2016/679)
