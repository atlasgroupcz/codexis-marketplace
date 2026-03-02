# Slovak General Court Decisions Search (SKVS)

Search for court decisions from Slovak general courts (okresne, krajske, etc.).

## cdx-sk Usage
Use `cdx-sk` for requests. It accepts standard curl flags and `cdx-sk://` URLs.

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS" \
  -H 'Content-Type: application/json' \
  -d '{"query": "náhrada škody", "limit": 5}'
```

## Endpoint

```
POST cdx-sk://search/SKVS
Content-Type: application/json
```

## Request Schema

```json
{
  "query": "string (optional) - fulltext search",
  "limit": "integer (1-100, default: 20)",
  "offset": "integer (default: 0)",

  "court": "string - filter by court code (e.g. OSBA1)",
  "courtName": "string - filter by court name (e.g. Okresný súd Bratislava I)",
  "judge": "string - filter by judge name (e.g. JUDr. Novák)",
  "spisovaZnacka": "string - filter by case file number (exact match, e.g. 1C/123/2024)",
  "decisionForm": "string - filter by decision form (e.g. Rozsudok)",
  "decisionNature": "string - filter by decision nature",
  "ecli": "string - filter by ECLI identifier",

  "dateFrom": "date (YYYY-MM-DD) - decision date from (inclusive)",
  "dateTo": "date (YYYY-MM-DD) - decision date to (inclusive)"
}
```

Query parameters (appended to URL, not in body):
- `sort` - `relevance | title | date` (default: relevance)
- `order` - `asc | desc` (default: desc)

## Response Schema

```json
{
  "results": [
    {
      "recordId": "ECLI_SK_OSBA1_2024_1234567890",
      "title": "Rozsudok - Okresný súd Bratislava I, sp. zn. 1C/123/2024, 2024-06-15",
      "court": "OSBA1",
      "courtName": "Okresný súd Bratislava I",
      "judge": "JUDr. Novák",
      "decisionForm": "Rozsudok",
      "decisionNature": "Prvostupňové nenapadnuté opravnými prostriedkami",
      "decisionDate": "2024-06-15",
      "spisovaZnacka": "1C/123/2024",
      "povodnaSpisovaZnacka": null,
      "ecli": "ECLI:SK:OSBA1:2024:1234567890",
      "legalDomain": "Občianske právo",
      "legalSubDomain": "Náhrada škody",
      "url": "https://www.slov-lex.sk/sudne-rozhodnutia/vseobecne-sudy-sr/...",
      "docId": "SKVS1234",
      "score": 12.45,
      "highlight": {
        "content_markdown": ["text with <mark>highlights</mark>"]
      },
      "docUrl": "cdx-sk://doc/SKVS1234/meta"
    }
  ],
  "totalResults": 4590000,
  "offset": 0,
  "limit": 20
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Document ID for retrieval (e.g. SKVS1234) |
| `recordId` | ECLI-based record identifier |
| `court` | Court code (e.g. OSBA1) |
| `courtName` | Full court name |
| `spisovaZnacka` | Case file number (spisovná značka) |
| `decisionForm` | Decision form (Rozsudok, Uznesenie, etc.) |
| `decisionNature` | Decision nature / procedural stage |
| `legalDomain` | Legal domain classification |

## Decision Forms (decisionForm facet)

- `Rozsudok` - Judgment
- `Uznesenie` - Resolution
- `Platobný rozkaz` - Payment order
- `Trestný rozkaz` - Penal order
- `Rozsudok v mene Slovenskej republiky` - Judgment in the name of the Slovak Republic

## Decision Nature (decisionNature facet)

- `Prvostupňové nenapadnuté opravnými prostriedkami` - First instance, not appealed
- `Prvostupňové právoplatné` - First instance, final
- `Odvolacie` - Appellate
- `Dovolacie` - Cassation

## Examples

### Search Court Decisions by Topic

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "náhrada škody",
    "decisionForm": "Rozsudok",
    "limit": 10
  }' | jq '.results[] | {docId, title, decisionDate}'
```

### Search by Case File Number

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS" \
  -H 'Content-Type: application/json' \
  -d '{
    "spisovaZnacka": "1C/123/2024",
    "limit": 5
  }' | jq '.results[] | {docId, spisovaZnacka, ecli}'
```

### Search by Court and Date Range

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "pracovný pomer",
    "courtName": "Okresný súd Bratislava I",
    "dateFrom": "2024-01-01",
    "dateTo": "2024-12-31",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .decisionDate}'
```

### Search by Judge, Sorted by Date

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS?sort=date&order=desc" \
  -H 'Content-Type: application/json' \
  -d '{
    "judge": "JUDr. Novák",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .decisionDate, court: .courtName}'
```

### Search by ECLI

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS" \
  -H 'Content-Type: application/json' \
  -d '{
    "ecli": "ECLI:SK:OSBA1:2024:1234567890",
    "limit": 5
  }' | jq '.results[] | {docId, ecli, title}'
```

## Working with Court Decision Documents

Documents are single-version. Text supports `page` param for page-level retrieval and `part` param for section-level retrieval.

```bash
DOC_ID="SKVS1234"

# Get full text
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text"

# Get specific page
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?page=1"

# Get available sections
cdx-sk -s "cdx-sk://doc/${DOC_ID}/parts" | jq '.parts[] | {id, oznacenie}'

# Get specific section (e.g. just the ruling)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?part=section-1"

# Get multiple sections
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?part=section-1&part=section-2"

# Get metadata
cdx-sk -s "cdx-sk://doc/${DOC_ID}/meta" | jq '{docId, url, metadata}'

# Get versions (single version for court decisions)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/versions" | jq '.'
```

Typical sections (from `/parts`): `Výrok` (ruling), `Odôvodnenie` (reasoning), `Poučenie` (legal instruction).

### Find Related Documents

```bash
cdx-sk -s "cdx-sk://doc/SKVS1234/related?type=REFERENCED_LAW&limit=10" | \
  jq '.results[] | {docId, title}'
```

Applicable relation types:
- `REFERENCED_LAW` - Laws referenced by this court decision
