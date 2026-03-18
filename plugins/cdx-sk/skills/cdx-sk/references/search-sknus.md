# Slovak Supreme & Constitutional Court Decisions Search (SKNUS)

Search for Slovak Supreme Court (NSSR) and Constitutional Court (USSR) decisions from www.slov-lex.sk. ~2,900 decisions total, dataset spans 1993-2018.

## cdx-sk Usage
Use `cdx-sk` for requests. It accepts standard curl flags and `cdx-sk://` URLs.

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS" \
  -H 'Content-Type: application/json' \
  -d '{"query": "zakladne prava", "limit": 5}'
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

  "court": "string - court code filter (NSSR or USSR)",
  "courtName": "string - exact court name (e.g. Najvyssi sud Slovenskej republiky)",
  "typRozhodnutia": "string - decision type in Slovak (e.g. Uznesenie, Nalez)",
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
      "title": "Uznesenie - K otazke pripustnosti dovolania",
      "court": "NSSR",
      "courtName": "Najvyssi sud Slovenskej republiky",
      "decisionType": "Uznesenie",
      "decisionDate": "2017-06-13",
      "caseNumber": "3Obdo/27/2018",
      "ecli": "ECLI:SK:NSSR:2017:2013200459",
      "legalDomains": ["Obcianske pravo", "Obchodne pravo"],
      "legalSentence": "Pravna veta rozhodnutia...",
      "docId": "SKNUS5678",
      "highlight": {
        "legalSentence": ["text with <mark>highlights</mark>"],
        "content_markdown": ["matching <mark>fragment</mark>"]
      },
      "docUrl": "cdx-sk://doc/SKNUS5678/meta"
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
| `docId` | Document ID for retrieval (e.g. SKNUS5678) |
| `court` | Court code: NSSR (Supreme) or USSR (Constitutional) |
| `caseNumber` | Case file number (spisovana znacka) |
| `ecli` | European Case Law Identifier |
| `legalDomains` | Legal domain classifications (array) |
| `legalSentence` | Key legal principle (pravna veta) |
| `decisionDate` | Date of the decision (YYYY-MM-DD) |
| `highlight` | Search snippets with `<mark>` tags on legalSentence and content_markdown |

## Courts (court filter)

- `NSSR` - Najvyssi sud Slovenskej republiky (Supreme Court, ~715 decisions)
- `USSR` - Ustavny sud Slovenskej republiky (Constitutional Court, ~2,189 decisions)

## Decision Types (typRozhodnutia filter)

- `Uznesenie` - Resolution
- `Nalez` - Finding (Constitutional Court)
- `Rozsudok` - Judgment

## Examples

### Search Constitutional Court Decisions

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "zakladne prava a slobody",
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

### Search by Date Range, Sorted by Date

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS?sort=date&order=desc" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "nahrada skody",
    "court": "NSSR",
    "dateFrom": "2015-01-01",
    "dateTo": "2018-12-31",
    "limit": 10
  }' | jq '.results[] | {docId, title, date: .decisionDate}'
```

### Search by ECLI

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS" \
  -H 'Content-Type: application/json' \
  -d '{
    "ecli": "ECLI:SK:NSSR:2017:2013200459",
    "limit": 1
  }' | jq '.results[0] | {docId, title, caseNumber}'
```

### Extract Legal Sentences

```bash
cdx-sk -s -X POST "cdx-sk://search/SKNUS" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "bezduvodne obohatenie",
    "court": "NSSR",
    "limit": 5
  }' | jq -r '.results[] | "## \(.title)\n\(.legalSentence)\n"'
```

## Working with SKNUS Documents

Documents are single-version (no timecutId needed). Text supports `page` param for page-level retrieval and `part` param for section-level retrieval.

```bash
DOC_ID="SKNUS5678"

# Get metadata (includes vazbyNaPredpisZbierky with resolved cdx-sk:// law links)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/meta" | jq '.'

# Get full text
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text"

# Get specific page
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?page=1"

# Get versions (single version for court decisions)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/versions" | jq '.'
```

### Retrieve Specific Sections

Sections typically include Vyrok (operative part) and Odovodnenie (reasoning). Use `/parts` to discover available section IDs, then pass them to `/text?part=...`:

```bash
# List sections
cdx-sk -s "cdx-sk://doc/SKNUS5678/parts" | jq '.parts[] | {id, oznacenie}'

# Get specific section
cdx-sk -s "cdx-sk://doc/SKNUS5678/text?part=section-1"

# Get multiple sections
cdx-sk -s "cdx-sk://doc/SKNUS5678/text?part=section-1&part=section-2"
```

### Download Attachments

```bash
cdx-sk -s "cdx-sk://doc/SKNUS5678/meta" | jq '.assets[] | {original_name, download_url}'

# Download a specific attachment
cdx-sk -s "cdx-sk://doc/SKNUS5678/attachment/content_1.pdf" --output decision.pdf
```

### Find Related Documents

```bash
# Get relation counts first
cdx-sk -s "cdx-sk://doc/SKNUS5678/related/counts" | jq '.'

# Get laws referenced by this decision
cdx-sk -s "cdx-sk://doc/SKNUS5678/related?type=REFERENCED_LAW&limit=10" | \
  jq '.results[] | {docId, title, docUrl}'
```

Applicable relation types for court decisions:
- `REFERENCED_LAW` - Laws referenced by this decision
