# Czech Municipal Regulations Search (CZSB)

Search for Czech municipal regulations (obecne zavazne vyhlasky, narizeni) and other acts from sbirkapp.gov.cz — the official collection of legal regulations of local self-governing units.

## cdx-cz-spp Usage
Use `cdx-cz-spp` for requests. It accepts standard curl flags and `cdx-cz-spp://` URLs.

```bash
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB" \
  -H 'Content-Type: application/json' \
  -d '{"query": "odpad", "limit": 5}'
```

## Endpoint

```
POST cdx-cz-spp://search/CZSB
Content-Type: application/json
```

## Request Schema

```json
{
  "query": "string (optional) - fulltext search in title, content, legal area, legal authorization",
  "limit": "integer (1-100, default: 20)",
  "offset": "integer (default: 0)",

  "druhPredpisu": "string - document type (e.g. 'Obecne zavazna vyhlaska')",
  "publikujici": "string - publisher/municipality (e.g. 'Mesto Liberec')",
  "oblastPravniUpravy": "string - legal area (e.g. 'Odpady')",
  "platnost": "string - validity status (e.g. 'Platne')",
  "cisloPredpisu": "string - document number (e.g. '1/2026')",
  "ico": "string - municipality ICO (exact match, e.g. '44992785')",
  "zakonneZmocneni": "string - legal authorization filter",
  "hlavniTyp": "string - document category: 'pp' (pravni predpisy) or 'oa' (ostatni akty)",

  "datumVydaniFrom": "date (YYYY-MM-DD) - issue date from",
  "datumVydaniTo": "date (YYYY-MM-DD) - issue date to",
  "datumZverejneniFrom": "date (YYYY-MM-DD) - publication date from",
  "datumZverejneniTo": "date (YYYY-MM-DD) - publication date to",
  "datumUcinnostiFrom": "date (YYYY-MM-DD) - effective date from",
  "datumUcinnostiTo": "date (YYYY-MM-DD) - effective date to"
}
```

Query parameters (appended to URL, not in body):
- `sort` - `relevance | title | date` (default: relevance)
- `order` - `asc | desc` (default: desc)

## Response Schema

Results are page-level — the same regulation may appear multiple times (once per matching page), sorted by relevance.

```json
{
  "results": [
    {
      "docId": "CZSB123",
      "hlavniTyp": "pp",
      "nazev": "o mistnim poplatku za obecni system odpadoveho hospodarstvi",
      "druhPredpisu": "Obecne zavazna vyhlaska",
      "publikujici": "Statutarni mesto Brno",
      "cisloPredpisu": "8/2025",
      "oblastPravniUpravy": "mistni poplatek",
      "datumVydani": "18.12.2025",
      "datumZverejneni": "14.01.2026 13:52",
      "datumNabytiUcinnosti": "01.02.2026",
      "platnost": "Platne",
      "pageNumber": 0,
      "sourceFile": "content_1",
      "highlight": {
        "content_markdown": ["text with <mark>odpad</mark> highlights"]
      },
      "docUrl": "cdx-cz-spp://doc/CZSB123",
      "pageUrl": "cdx-cz-spp://doc/CZSB123/attachment/content_1.pdf#page=1"
    }
  ],
  "totalResults": 1542,
  "offset": 0,
  "limit": 20
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Document ID for all `/doc/{docId}/*` endpoints |
| `hlavniTyp` | `pp` = pravni predpisy (regulations), `oa` = ostatni akty (other acts) |
| `druhPredpisu` | Document type (Obecne zavazna vyhlaska, Narizeni, etc.) |
| `platnost` | Validity status (Platne = valid) |
| `pageNumber` | Matching page number (0-based, from AIMD conversion) |
| `sourceFile` | Source file for the matching page |
| `highlight` | Search hit highlights with `<mark>` tags |
| `docUrl` | cdx-cz-spp:// link to document |
| `pageUrl` | cdx-cz-spp:// link to matching page in attachment |
| `totalResults` | Total matching pages (not regulations — same doc may contribute multiple page hits) |

## Document Types (druhPredpisu filter)

PP types:
- `Obecne zavazna vyhlaska` - Municipal ordinance
- `Narizeni` - Regulation

OA types (hlavniTyp=oa):
- `Nalez Ustavniho soudu` - Constitutional Court ruling
- `Rozhodnuti o pozastaveni ucinnosti` - Decision on suspension
- `Smlouva` - Contract
- `Stav nebezpeci` - State of emergency

## Examples

### Search Municipal Regulations by Topic

```bash
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "odpadove hospodarstvi",
    "hlavniTyp": "pp",
    "limit": 10
  }' | jq '.results[] | {docId, nazev, publikujici, cisloPredpisu}'
```

### Search by Publisher

```bash
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB" \
  -H 'Content-Type: application/json' \
  -d '{
    "publikujici": "Statutarni mesto Brno",
    "platnost": "Platne",
    "limit": 10
  }' | jq '.results[] | {docId, nazev, cisloPredpisu, platnost}'
```

### Search by Date Range

```bash
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "mistni poplatek",
    "datumVydaniFrom": "2025-01-01",
    "datumVydaniTo": "2025-12-31",
    "limit": 10
  }' | jq '.results[] | {docId, nazev, datumVydani}'
```

### Sort by Date (Most Recent First)

```bash
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB?sort=date&order=desc" \
  -H 'Content-Type: application/json' \
  -d '{
    "hlavniTyp": "pp",
    "limit": 10
  }' | jq '.results[] | {docId, nazev, datumVydani, publikujici}'
```

### Find by Document Number

```bash
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB" \
  -H 'Content-Type: application/json' \
  -d '{
    "cisloPredpisu": "1/2026",
    "limit": 5
  }' | jq '.results[] | {docId, nazev, publikujici}'
```

## Working with Sbirkapp Documents

```bash
DOC_ID="CZSB123"

# Get document metadata
cdx-cz-spp -s "cdx-cz-spp://doc/${DOC_ID}/meta" | jq '.'

# Get full text
cdx-cz-spp -s "cdx-cz-spp://doc/${DOC_ID}/text"

# Get TOC
cdx-cz-spp -s "cdx-cz-spp://doc/${DOC_ID}/toc" | jq '.'

# Get versions
cdx-cz-spp -s "cdx-cz-spp://doc/${DOC_ID}/versions" | jq '.'
```

### Versioning

Documents can have multiple versions. Use `/versions` to list them:

```bash
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/versions" | jq '.versions[] | {versionNum, nazev, assets}'
```

Use the `version` query param on `/text` to retrieve a specific version's text:

```bash
# Get text for a specific version
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/text?version=1"
```

### Text Retrieval Options

The `/text` endpoint supports several parameters:

```bash
# Get a specific page (0-based)
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/text?page=0"

# Get a specific part by ID
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/text?part=section-1"

# Get a specific file's text
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/text?file=content_1"
```

### Parts (Sections)

Use `/parts` to list navigable sections, then retrieve specific parts:

```bash
# List parts
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/parts" | jq '.parts[] | {id, heading, level}'

# Get text of a specific part
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/text?part=section-1"
```

### Find Related Documents

```bash
# Documents this regulation cancels
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/related?type=CANCELS&limit=10" | \
  jq '.results[] | {sourceId, nazev, cisloPredpisu}'

# Documents that amend this regulation
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/related?type=AMENDED_BY&limit=10" | \
  jq '.results[] | {sourceId, nazev}'

# Get counts of all relation types
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/related/counts" | jq '.'
```

Relation types:
- `CANCELS` - Regulations this document cancels (Zruseni)
- `CANCELLED_BY` - Regulations that cancel this document
- `AMENDS` - Regulations this document amends (Novelizace)
- `AMENDED_BY` - Regulations that amend this document
- `IMPLEMENTS` - Regulations this document implements (Provedeni)
- `IMPLEMENTED_BY` - Implementing regulations
- `OTHER` - Other relationships
