# Slovak General Court Decisions Search (SKVS)

Search for decisions from Slovak general courts (okresne, krajske, and other general courts). ~4.6M decisions.

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
  "query": "string (optional) - fulltext search on decision text",
  "limit": "integer (1-100, default: 20)",
  "offset": "integer (default: 0)",

  "court": "string - court code exact match (e.g. 'OSBA1')",
  "courtName": "string - court name exact match (e.g. 'Okresny sud Bratislava I')",
  "judge": "string - judge name exact match (e.g. 'JUDr. Novak')",
  "spisovaZnacka": "string - case file number exact match (e.g. '1C/123/2024')",
  "decisionForm": "string - decision form (e.g. 'Rozsudok')",
  "decisionNature": "string - decision nature filter",
  "ecli": "string - ECLI identifier exact match",
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
      "docId": "SKVS5678",
      "highlight": {
        "content_markdown": ["text with <em>highlights</em>"]
      },
      "docUrl": "cdx-sk://doc/SKVS5678/meta"
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
| `docId` | Document ID for retrieval (e.g. SKVS5678) |
| `recordId` | Internal record identifier (ECLI-based) |
| `court` | Court code (e.g. OSBA1, KSBA) |
| `courtName` | Full court name in Slovak |
| `spisovaZnacka` | Case file number (spisovna znacka) |
| `povodnaSpisovaZnacka` | Original case file number (if case was transferred) |
| `ecli` | European Case Law Identifier |
| `legalDomain` / `legalSubDomain` | Legal domain classification |
| `decisionForm` | Type of decision (Rozsudok, Uznesenie, etc.) |
| `decisionNature` | Nature of the decision (first-instance, appellate, etc.) |

## Decision Forms (decisionForm filter)

- `Rozsudok` - Judgment
- `Uznesenie` - Resolution
- `Platobny rozkaz` - Payment order
- `Zmenkovy platobny rozkaz` - Bill of exchange payment order

## Decision Nature (decisionNature filter)

- `Prvostupnove nenapadnute opravnymi prostriedkami` - First-instance not challenged
- `Odvolacie` - Appellate
- `Dovolacie` - Cassation

## Examples

### Search by Topic

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
DOC_ID="SKVS5678"

# Get metadata
cdx-sk -s "cdx-sk://doc/${DOC_ID}/meta" | jq '.'

# Get full text
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text"

# Get specific page
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?page=1"

# Get versions (single version for court decisions)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/versions" | jq '.'

# Download attachment
cdx-sk -s "cdx-sk://doc/${DOC_ID}/attachment/content_1.pdf" -o decision.pdf
```

### Retrieve Specific Sections

Use `/parts` to discover section IDs, then pass them to `/text?part=...`:

```bash
# List sections
cdx-sk -s "cdx-sk://doc/SKVS5678/parts" | jq '.parts[] | {id, oznacenie}'

# Get specific section
cdx-sk -s "cdx-sk://doc/SKVS5678/text?part=section-1"

# Get multiple sections
cdx-sk -s "cdx-sk://doc/SKVS5678/text?part=section-1&part=section-2"
```

### Find Related Documents

```bash
# Get relation counts first
cdx-sk -s "cdx-sk://doc/SKVS5678/related/counts" | jq '.'

# Get laws referenced by this decision
cdx-sk -s "cdx-sk://doc/SKVS5678/related?type=REFERENCED_LAW&limit=10" | \
  jq '.results[] | {docId, title}'
```

Applicable relation types for court decisions:
- `REFERENCED_LAW` - Laws referenced by this decision
