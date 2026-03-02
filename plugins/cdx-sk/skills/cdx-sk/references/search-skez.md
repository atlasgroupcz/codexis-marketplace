# Slovak Legislation Search (SKEZ)

Search for Slovak laws and regulations from e-Zbierka (Zbierka zakonov SR).

## cdx-sk Usage
Use `cdx-sk` for requests. It accepts standard curl flags and `cdx-sk://` URLs.

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{"query": "dan z prijmov", "limit": 5}'
```

## Endpoint

```
POST cdx-sk://search/SKEZ
Content-Type: application/json
```

## Request Schema

```json
{
  "query": "string (optional) - fulltext search in paragraph text, titles, and headings",
  "limit": "integer (1-100, default: 20)",
  "offset": "integer (default: 0)",

  "docNumber": "string - exact match on document number (e.g. '40/1964 Zb.')",
  "typ": "string - exact match on document type (e.g. 'Zakon')",
  "validAt": "date (YYYY-MM-DD) - filter to versions valid at this date",
  "issuedFrom": "date (YYYY-MM-DD) - declared date from (inclusive)",
  "issuedTo": "date (YYYY-MM-DD) - declared date to (inclusive)"
}
```

Query parameters (appended to URL, not in body):
- `sort` - `relevance | title | date` (default: relevance)
- `order` - `asc | desc` (default: desc)

## Response Schema

Runtime note: `/text` and `/toc` accept an optional `timecutId` parameter to select a specific version.
Search results return the best-matching version. Use `/versions` to list all available versions.

Results are collapsed by law -- one result per law, showing the best-matching version. `totalResults` reflects the count of unique laws, not individual versions.

```json
{
  "results": [
    {
      "recordId": "obciansky_zakonnik_40_1964",
      "title": "Obciansky zakonnik",
      "docNumber": "40/1964",
      "docType": "zakon",
      "author": "Narodne zhromazdenie Ceskoslovenskej socialistickej republiky",
      "declaredDate": "1964-03-26",
      "validFrom": "2025-01-01",
      "validTo": null,
      "vyhlasene": false,
      "url": "https://www.slov-lex.sk/ezbierky/pravne-predpisy/SK/ZZ/1964/40",
      "iri": "/SK/ZZ/1964/40",
      "legalDomains": ["Obcianske pravo"],
      "lawNumber": 40,
      "lawYear": 1964,
      "sectionOznacenie": "ss 123",
      "sectionNadpis": "Nadobudnutie vlastnictva",
      "docId": "SKEZ1234",
      "score": 12.45,
      "highlight": {
        "sectionText": ["text with <mark>highlights</mark>"]
      },
      "docUrl": "cdx-sk://doc/SKEZ1234/meta"
    }
  ],
  "totalResults": 156,
  "offset": 0,
  "limit": 20
}
```

## Key Fields Explained

| Field | Description |
|-------|-------------|
| `docId` | Document ID for `/doc/{docId}/*` endpoints |
| `recordId` | Law-level identifier (used for collapsing) |
| `docNumber` | Regulation number with suffix (e.g. "40/1964 Zb.") |
| `validFrom` / `validTo` | Validity period of the matched version (null validTo = currently valid) |
| `vyhlasene` | True if this is the as-published version |
| `lawNumber` / `lawYear` | Numeric law number and year (use with `/law/{number}/{year}`) |
| `sectionOznacenie` | Matched section designation (e.g. "ss 123") |
| `highlight` | Search snippets with `<mark>` tags |

## Document Types (typ filter)

- `Zakon` - Law
- `Vyhlaska` - Decree
- `Oznamenie` - Notice
- `Opatrenie` - Measure
- `Nariadenie vlady` - Government regulation

## Examples

### Search Slovak Laws

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "obciansky zakonnik",
    "typ": "Zakon",
    "limit": 10
  }' | jq '.results[] | {docId, title, docNumber}'
```

### Search Currently Valid Legislation

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "dan z prijmov",
    "validAt": "2026-01-01",
    "limit": 10
  }' | jq '.results[] | {docId, title, validFrom, validTo}'
```

### Search by Document Number

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{
    "docNumber": "311/2001 Z. z.",
    "limit": 5
  }' | jq '.results[] | {docId, title, docNumber}'
```

### Search Recent Legislation by Date

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ?sort=date&order=desc" \
  -H 'Content-Type: application/json' \
  -d '{
    "issuedFrom": "2025-01-01",
    "limit": 10
  }' | jq '.results[] | {docId, title, declaredDate}'
```

## Working with Ezbierka Documents

```bash
DOC_ID="SKEZ1234"

# Get metadata
cdx-sk -s "cdx-sk://doc/${DOC_ID}/meta" | jq '.'

# Get full text (defaults to currently valid version)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text"

# Get text of a specific version
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?timecutId=1964_40_2025-01-01"

# Get TOC
cdx-sk -s "cdx-sk://doc/${DOC_ID}/toc" | jq '.'

# Get all versions
cdx-sk -s "cdx-sk://doc/${DOC_ID}/versions" | jq '.versions[] | {versionId, validFrom, validTo, vyhlasene}'

# Get available paragraphs
cdx-sk -s "cdx-sk://doc/${DOC_ID}/parts" | jq '.parts[] | {id, oznacenie, nadpis}'

# Get specific paragraphs
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?part=paragraf-1&part=paragraf-2"
```

### Direct Access by Law Number/Year (Skip Search)

If you know the law number and year (e.g., 40/1964), resolve directly:

```bash
# Resolve law
cdx-sk -s "cdx-sk://law/SK/40/1964" | jq '{docId, title, docNumber}'

# Get law text directly
cdx-sk -s "cdx-sk://law/SK/40/1964/text"

# Get law TOC
cdx-sk -s "cdx-sk://law/SK/40/1964/toc" | jq '.'

# Get law versions
cdx-sk -s "cdx-sk://law/SK/40/1964/versions" | jq '.versions[]'

# Get law metadata
cdx-sk -s "cdx-sk://law/SK/40/1964/meta" | jq '.'
```

### Selecting a Specific Version

Laws have multiple versions (timecuts) with validity periods. Use `/versions` to list them, then pass `timecutId` to `/text` or `/toc`:

```bash
# List versions
cdx-sk -s "cdx-sk://doc/SKEZ1234/versions" | jq '.versions[] | {versionId, validFrom, validTo}'

# Get text of a specific historical version
cdx-sk -s "cdx-sk://doc/SKEZ1234/text?timecutId=1964_40_2020-01-01"
```

Without `timecutId`, the API defaults to the currently valid version (validFrom <= today <= validTo), falls back to latest by validFrom, then to the as-published version.

### Working with Parts (Paragraphs)

Parts represent individual paragraphs of a law. Use them for targeted extraction:

```bash
# List all parts with page references
cdx-sk -s "cdx-sk://doc/SKEZ1234/parts" | jq '.parts[] | {id, oznacenie, nadpis, startPage}'

# Get text of specific paragraphs
cdx-sk -s "cdx-sk://doc/SKEZ1234/text?part=paragraf-123&part=paragraf-124"
```

### Find Related Documents

```bash
# Get relation counts first
cdx-sk -s "cdx-sk://doc/SKEZ1234/related/counts" | jq '.'

# Get implementing regulations
cdx-sk -s "cdx-sk://doc/SKEZ1234/related?type=IMPLEMENTING&limit=10" | \
  jq '.results[] | {docId, title}'

# Get amending laws
cdx-sk -s "cdx-sk://doc/SKEZ1234/related?type=AMENDS&limit=10" | \
  jq '.results[] | {docId, title}'
```

Relation types:
- `IMPLEMENTING` - Implementing regulations (vykonacie predpisy)
- `AMENDS` - Laws amending this regulation
- `AMENDED_BY` - Laws amended by this regulation
- `REPEALS` - Laws repealed by this regulation
- `REFERENCING_DECISION` - Court decisions referencing this law
- `REFERENCED_LAW` - Laws referenced by a court decision
