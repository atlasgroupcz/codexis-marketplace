# Contract Templates Search (VS)

Search for contract specimens and templates.

## cdx Usage
Use `cdx` for requests. It is opinionated: it runs silently by default, and `-d` implies `POST` plus `Content-Type: application/json` unless you override them.

```bash
cdx "cdx://search/VS" \
  -d '{"query": "kupní smlouva", "limit": 5}'
```

## Endpoint

```
POST cdx://search/VS
JSON request body
```

## Request Schema

```json
{
  "query": "string (required) - fulltext search",
  "limit": "integer (1-50, default: 10)",
  "offset": "integer (default: 0)",
  "sort": "RELEVANCE | NAME | DATE",
  "sortOrder": "ASC | DESC (default: DESC)",

  "autor": ["AK Sokol Novák tdpA", "Linde Praha a. s.", ...],
  "kategorie": ["Pracovní právo", "Občanské právo hmotné", ...]
}
```

## Response Schema

```json
{
  "results": [
    {
      "docId": "VS1000018",
      "docUrl": "cdx://doc/VS1000018/text",
      "title": "Kupní smlouva",
      "snippet": "",
      "author": "CODEXIS publishing",
      "createdDate": "2025-07-01"
    }
  ],
  "totalResults": 178,
  "offset": 0,
  "limit": 3
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Document ID for retrieval |
| `docUrl` | Template text URL |
| `author` | Template author/law firm |
| `createdDate` | Creation/update date |

## Categories (kategorie facet)

- `Pracovní právo (vzory dle zákoníku práce)` - Labor law
- `Občanské právo hmotné (vzory dle občanského zákoníku)` - Civil law
- `Obchodní právo` - Commercial law
- `Správní právo` - Administrative law

## Examples

### Search Purchase Agreements

```bash
cdx "cdx://search/VS" \
  -d '{
    "query": "kupní smlouva",
    "limit": 10
  }' | jq '.results[] | {docId, title, author}'
```

### Search Employment Contracts

```bash
cdx "cdx://search/VS" \
  -d '{
    "query": "pracovní smlouva",
    "kategorie": ["Pracovní právo (vzory dle zákoníku práce)"],
    "limit": 10
  }' | jq '.results[] | {docId, title}'
```

### Search Lease Agreements

```bash
cdx "cdx://search/VS" \
  -d '{
    "query": "nájemní smlouva",
    "limit": 10
  }' | jq '.results[] | {docId, title, author}'
```

### Search by Author

```bash
cdx "cdx://search/VS" \
  -d '{
    "query": "*",
    "autor": ["CODEXIS publishing"],
    "limit": 20
  }' | jq '.results[] | {docId, title}'
```

## Working with Templates

### Get Template Text

Templates do not have TOC (`/toc` currently returns HTTP 500) - fetch full text:

```bash
DOC_ID="VS1000018"
cdx "cdx://doc/${DOC_ID}/text"
```

### Save Template to File

```bash
DOC_ID="VS1000018"
cdx "cdx://doc/${DOC_ID}/text" > template.txt
```

### Find Related Legislation

```bash
cdx "cdx://doc/VS1000018/related?type=SOUVISEJICI_LEGISLATIVA_CR" | \
  jq '.results[] | {docId, title}'
```

## Common Template Types

| Query | Description |
|-------|-------------|
| `kupní smlouva` | Purchase agreement |
| `nájemní smlouva` | Lease agreement |
| `pracovní smlouva` | Employment contract |
| `smlouva o dílo` | Work contract |
| `plná moc` | Power of attorney |
| `dohoda o provedení práce` | Agreement on work performance |
| `výpověď` | Termination notice |
| `odstoupení od smlouvy` | Contract withdrawal |
