# Slovak Supreme & Constitutional Court Decisions Search (SKNUS)

Search for Slovak Supreme Court (NSSR) and Constitutional Court (USSR) decisions.

## cdx-sk Usage
Use `cdx-sk` for requests. It accepts standard curl flags and `cdx-sk://` URLs.

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS" \
  -H 'Content-Type: application/json' \
  -d '{"query": "základné práva", "limit": 5}'
```

## Endpoint

```
POST cdx-sk://search/SKNUS
Content-Type: application/json
```

## Request Schema

```json
{
  "query": "string (optional) - fulltext search across legal sentences and decision text",
  "limit": "integer (1-100, default: 20)",
  "offset": "integer (default: 0)",

  "court": "string - court code filter (e.g. NSSR, USSR)",
  "courtName": "string - court name filter (exact match)",
  "typRozhodnutia": "string - decision type in Slovak (e.g. Uznesenie, Nález)",
  "decisionType": "string - alias for typRozhodnutia",
  "spisovaZnacka": "string - case file number (e.g. 3Obdo/27/2018)",
  "caseNumber": "string - alias for spisovaZnacka",
  "ecli": "string - ECLI identifier (e.g. ECLI:SK:NSSR:2017:2013200459)",

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
      "recordId": "ECLI_SK_NSSR_2017_2013200459",
      "title": "Uznesenie - K otázke prípustnosti dovolania",
      "court": "NSSR",
      "courtName": "Najvyšší súd Slovenskej republiky",
      "decisionType": "Uznesenie",
      "decisionDate": "2017-06-13",
      "caseNumber": "3Obdo/27/2018",
      "ecli": "ECLI:SK:NSSR:2017:2013200459",
      "legalDomains": ["Občianske právo", "Obchodné právo"],
      "legalSentence": "Právna veta rozhodnutia...",
      "url": "https://www.slov-lex.sk/sudne-rozhodnutia/judikaty/abc-123",
      "docId": "SKNUS1234",
      "score": 12.45,
      "highlight": {
        "legalSentence": ["text with <mark>highlights</mark>"],
        "content_markdown": ["matching <mark>fragment</mark>"]
      },
      "docUrl": "cdx-sk://doc/SKNUS1234/meta"
    }
  ],
  "totalResults": 2904,
  "offset": 0,
  "limit": 20
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Document ID for retrieval (e.g. SKNUS1234) |
| `court` | Court code: NSSR (Supreme) or USSR (Constitutional) |
| `caseNumber` | Case file number (spisovná značka) |
| `ecli` | European Case Law Identifier |
| `legalDomains` | Legal domain classifications |
| `legalSentence` | Key legal principle (právna veta) |
| `decisionDate` | Date of the decision |

## Courts (court facet)

- `NSSR` - Najvyšší súd Slovenskej republiky (Supreme Court)
- `USSR` - Ústavný súd Slovenskej republiky (Constitutional Court)

## Decision Types (typRozhodnutia facet)

- `Uznesenie` - Resolution
- `Nález` - Finding (Constitutional Court)
- `Rozsudok` - Judgment

## Examples

### Search Constitutional Court Decisions

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "základné práva a slobody",
    "court": "USSR",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .decisionDate, ecli}'
```

### Search by Case File Number

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS" \
  -H 'Content-Type: application/json' \
  -d '{
    "spisovaZnacka": "3Obdo/27/2018",
    "limit": 5
  }' | jq '.results[] | {docId, caseNumber, ecli}'
```

### Search Supreme Court Decisions by Date Range

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "náhrada škody",
    "court": "NSSR",
    "dateFrom": "2015-01-01",
    "dateTo": "2018-12-31",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .decisionDate}'
```

### Sort by Date

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS?sort=date&order=desc" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "vlastnícke právo",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .decisionDate}'
```

### Extract Legal Sentences

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "bezdôvodné obohatenie",
    "court": "NSSR",
    "limit": 5
  }' | jq -r '.results[] | "## \(.title)\n\(.legalSentence)\n"'
```

## Working with SKNUS Documents

Documents are single-version (no timecutId needed). Text supports `page` param for page-level retrieval and `part` param for section-level retrieval.

```bash
DOC_ID="SKNUS1234"

# Get full text
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text"

# Get specific page
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?page=1"

# Get TOC / available sections
cdx-sk -s "cdx-sk://doc/${DOC_ID}/parts" | jq '.'

# Get metadata
cdx-sk -s "cdx-sk://doc/${DOC_ID}/meta" | jq '.'

# Get versions (single version for court decisions)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/versions" | jq '.'
```

### Retrieve Specific Sections

Sections typically include Výrok (operative part) and Odôvodnenie (reasoning). Use `/parts` to discover available section IDs, then pass them to `/text?part=...`:

```bash
# List sections
cdx-sk -s "cdx-sk://doc/SKNUS1234/parts" | jq '.parts[] | {id, oznacenie}'

# Get specific section
cdx-sk -s "cdx-sk://doc/SKNUS1234/text?part=section-1"

# Get multiple sections
cdx-sk -s "cdx-sk://doc/SKNUS1234/text?part=section-1&part=section-2"
```

### Find Related Documents

```bash
cdx-sk -s "cdx-sk://doc/SKNUS1234/related?type=REFERENCED_LAW&limit=10" | \
  jq '.results[] | {docId, title}'
```

Applicable relation types for court decisions:
- `REFERENCED_LAW` - Laws referenced by this decision
