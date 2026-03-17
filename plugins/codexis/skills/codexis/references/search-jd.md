# Czech Case Law Search (JD)

Search for judicial decisions from Czech courts.

## cdx Usage
Use `cdx` for requests. It is opinionated: it runs silently by default, and `-d` implies `POST` plus `Content-Type: application/json` unless you override them.

```bash
cdx "cdx://search/JD" \
  -d '{"query": "náhrada škody", "limit": 5}'
```

## Endpoint

```
POST cdx://search/JD
JSON request body
```

## Request Schema

```json
{
  "query": "string (required) - fulltext search",
  "limit": "integer (1-50, default: 10)",
  "offset": "integer (default: 0)",
  "sort": "RELEVANCE | CITEX | DATE | NAME",
  "sortOrder": "ASC | DESC (default: DESC)",

  "soud": ["Nejvyšší soud", "Ústavní soud", ...],
  "mesto": ["Brno", "Praha", ...],
  "typ": ["Rozsudek", "Usnesení", "Nález", ...],

  "issuedFrom": "date (YYYY-MM-DD)",
  "issuedTo": "date"
}
```

## Response Schema

```json
{
  "results": [
    {
      "docId": "JD252461",
      "docUrl": "cdx://doc/JD252461/text",
      "title": "Nález - Ke stanovení výše náhrady škody za neoprávněně odebranou elektřinu",
      "snippet": "text with <mark>highlights</mark>",
      "legalSentence": "<div>Legal principle HTML...</div>",
      "createdDate": "2015-08-11",
      "court": "Ústavní soud",
      "source": "Sbírka nálezů a usnesení ÚS",
      "city": "Brno",
      "docType": "Nález",
      "spZns": ["I. ÚS 668/15"],
      "cislaJednaci": ["I. ÚS 668/15-1"],
      "sbirkoveCislo": "SbNU 141/78",
      "ecli": "ECLI:CZ:US:2015:1.US.668.15.1",
      "note": null,
      "derogated": false
    }
  ],
  "totalResults": 193961,
  "offset": 0,
  "limit": 3
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Document ID for retrieval |
| `docUrl` | Decision text URL |
| `legalSentence` | Key legal principle (HTML formatted) |
| `court` | Court name |
| `spZns` | Case file numbers (spisové značky) |
| `cislaJednaci` | Reference numbers |
| `ecli` | European Case Law Identifier |
| `sbirkoveCislo` | Collection number |
| `derogated` | True if decision was overturned |

## Courts (soud facet)

Main courts:
- `Ústavní soud` - Constitutional Court
- `Nejvyšší soud` - Supreme Court
- `Nejvyšší správní soud` - Supreme Administrative Court
- `Vrchní soud v Praze` - High Court in Prague
- `Vrchní soud v Olomouci` - High Court in Olomouc
- `Krajský soud v ...` - Regional courts

## Decision Types (typ facet)

- `Nález` - Finding (Constitutional Court)
- `Rozsudek` - Judgment
- `Usnesení` - Resolution
- `Stanovisko` - Opinion

## Examples

### Search Constitutional Court Decisions

```bash
cdx "cdx://search/JD" \
  -d '{
    "query": "svoboda projevu",
    "soud": ["Ústavní soud"],
    "typ": ["Nález"],
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .createdDate, ecli}'
```

### Search by Case File Number

```bash
cdx "cdx://search/JD" \
  -d '{
    "query": "I. ÚS 668/15",
    "limit": 5
  }' | jq '.results[] | {docId, spZns, ecli}'
```

### Search Recent Supreme Court Decisions

```bash
cdx "cdx://search/JD" \
  -d '{
    "query": "odpovědnost za škodu",
    "soud": ["Nejvyšší soud"],
    "issuedFrom": "2024-01-01",
    "sort": "DATE",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .createdDate}'
```

### Search Administrative Court Decisions

```bash
cdx "cdx://search/JD" \
  -d '{
    "query": "stavební povolení",
    "soud": ["Nejvyšší správní soud"],
    "limit": 10
  }' | jq '.results[] | {docId, title, court}'
```

### Extract Legal Sentences (Právní věty)

```bash
cdx "cdx://search/JD" \
  -d '{
    "query": "bezdůvodné obohacení",
    "soud": ["Nejvyšší soud"],
    "limit": 5
  }' | jq '.results[] | {docId, legalSentence}' | \
  sed 's/<[^>]*>//g'  # Strip HTML tags
```

### Search by ECLI

```bash
cdx "cdx://search/JD" \
  -d '{
    "query": "ECLI:CZ:US:2015",
    "limit": 10
  }' | jq '.results[] | {docId, ecli, title}'
```

## Processing Results

### Get Document Text

JD documents often expose a lightweight generated TOC (`SEKCE...` element IDs). Prefer TOC + marker extraction when available; otherwise use full text directly:

```bash
DOC_ID="JD252461"
cdx "cdx://doc/${DOC_ID}/toc" | jq '.'
cdx "cdx://doc/${DOC_ID}/text"
```

### Extract Clean Legal Sentences

```bash
cdx "cdx://search/JD" \
  -d '{"query": "náhrada škody", "limit": 3}' | \
  jq -r '.results[] | "## \(.title)\n\(.legalSentence)\n"' | \
  sed 's/<[^>]*>//g'
```

### Group by Court

```bash
cdx "cdx://search/JD" \
  -d '{"query": "pracovní úraz", "limit": 50}' | \
  jq '.results | group_by(.court) | map({court: .[0].court, count: length}) | sort_by(-.count)'
```

### Find Related Legislation

Case law often references legislation. After finding a decision, use the relations endpoint:

```bash
cdx "cdx://doc/JD252461/related?type=SOUVISEJICI_LEGISLATIVA_CR&limit=10" | \
  jq '.results[] | {docId, title}'
```
