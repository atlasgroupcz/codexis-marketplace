# Slovak General Court Decisions Search (SKVS)

Search for decisions from Slovak general courts (okresne, krajske, etc.).

## cdx-sk Usage
Use `cdx-sk` for requests. It accepts standard curl flags and `cdx-sk://` URLs.

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS" \
  -H 'Content-Type: application/json' \
  -d '{"query": "nahrada skody", "limit": 5}'
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

  "court": "string - filter by court code (exact match, e.g. 'OSBA1')",
  "courtName": "string - filter by court name (e.g. 'Okresny sud Bratislava I')",
  "judge": "string - filter by judge name (e.g. 'JUDr. Novak')",
  "spisovaZnacka": "string - filter by case file number (exact match, e.g. '1C/123/2024')",
  "decisionForm": "string - filter by decision form (e.g. 'Rozsudok')",
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
      "title": "Rozsudok - Okresny sud Bratislava I, sp. zn. 1C/123/2024, 2024-06-15",
      "court": "OSBA1",
      "courtName": "Okresny sud Bratislava I",
      "judge": "JUDr. Novak",
      "decisionForm": "Rozsudok",
      "decisionNature": "Prvostupnove nenapadnute opravnymi prostriedkami",
      "decisionDate": "2024-06-15",
      "spisovaZnacka": "1C/123/2024",
      "povodnaSpisovaZnacka": null,
      "ecli": "ECLI:SK:OSBA1:2024:1234567890",
      "legalDomain": "Obcianske pravo",
      "legalSubDomain": "Nahrady",
      "docId": "SKVS1234",
      "score": 12.34,
      "highlight": {
        "content_markdown": ["text with <em>highlights</em>"]
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
| `recordId` | Internal record identifier (ECLI-based) |
| `court` | Court code (e.g. OSBA1, KSBA) |
| `courtName` | Full court name in Slovak |
| `spisovaZnacka` | Case file number (spisovna znacka) |
| `ecli` | European Case Law Identifier |
| `legalDomain` | Legal domain classification |
| `decisionForm` | Type of decision (Rozsudok, Uznesenie, etc.) |

## Decision Forms (decisionForm facet)

- `Rozsudok` - Judgment
- `Uznesenie` - Resolution
- `Platobny rozkaz` - Payment order
- `Zmenkovy platobny rozkaz` - Bill of exchange payment order

## Decision Nature (decisionNature facet)

- `Prvostupnove nenapadnute opravnymi prostriedkami` - First-instance not challenged
- `Prvostupnove nenapadnute opravnymi prostriedkami - Loss` - First-instance not challenged (loss)
- `Odvolacie` - Appellate
- `Dovolacie` - Cassation

## Examples

### Search Court Decisions by Topic

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "nahrada skody",
    "limit": 10
  }' | jq '.results[] | {docId, title, decisionDate, court}'
```

### Filter by Court and Date Range

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "pracovny pomer",
    "courtName": "Okresny sud Bratislava I",
    "dateFrom": "2024-01-01",
    "dateTo": "2024-12-31",
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

### Sort by Date (Most Recent First)

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS?sort=date&order=desc" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "bezduvodne obohatenie",
    "limit": 10
  }' | jq '.results[] | {docId, title, decisionDate}'
```

### Filter by Decision Form

```bash
cdx-sk -s -X POST "cdx-sk://search/SKVS" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "vlastnicke pravo",
    "decisionForm": "Rozsudok",
    "limit": 10
  }' | jq '.results[] | {docId, title, decisionForm, courtName}'
```

## Working with SKVS Documents

Documents are single-version (no timecutId). Text supports `page` param for page-level retrieval. `/parts` returns section items with id and oznacenie (e.g., "Vyrok", "Odovodnenie").

```bash
DOC_ID="SKVS1234"

# Get full text
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text"

# Get specific page
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?page=1"

# Get TOC / sections
cdx-sk -s "cdx-sk://doc/${DOC_ID}/parts" | jq '.'

# Get specific section text
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?part=section-1"

# Get multiple sections
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?part=section-1&part=section-2"

# Get metadata
cdx-sk -s "cdx-sk://doc/${DOC_ID}/meta" | jq '.'

# Get versions (single version expected for court decisions)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/versions" | jq '.'

# Download attachment
cdx-sk -s "cdx-sk://doc/${DOC_ID}/attachment/content_1.pdf" -o decision.pdf
```

### Find Related Documents

```bash
cdx-sk -s "cdx-sk://doc/SKVS1234/related?type=REFERENCED_LAW&limit=10" | \
  jq '.results[] | {docId, title}'
```

Applicable relation types for court decisions:
- `REFERENCED_LAW` - Laws referenced by this court decision
- `REFERENCING_DECISION` - Other court decisions referencing the same laws
