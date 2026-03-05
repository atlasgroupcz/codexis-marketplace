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
  "query": "string (optional) - fulltext search in paragraph text, titles, headings",
  "limit": "integer (1-100, default: 20)",
  "offset": "integer (default: 0)",

  "docNumber": "string - exact match on document number (e.g. '40/1964 Zb.')",
  "typ": "string - exact document type (e.g. 'Zakon')",
  "validAt": "date (YYYY-MM-DD) - filter to versions valid at this date",
  "issuedFrom": "date (YYYY-MM-DD) - declared date from (inclusive)",
  "issuedTo": "date (YYYY-MM-DD) - declared date to (inclusive)"
}
```

Query parameters (appended to URL, not in body):
- `sort` - `relevance | title | date` (default: relevance)
- `order` - `asc | desc` (default: desc)

## Response Schema

Runtime note: `/text` and `/toc` require a version-specific `timecutId` parameter or default to the currently valid version.
Search results return the best-matching version. Use `/versions` to list all available versions.

Results are collapsed by law -- one result per law, showing the best-matching version.

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
      "legalDomains": ["Obcianske pravo"],
      "lawNumber": 40,
      "lawYear": 1964,
      "sectionOznacenie": "SS 123",
      "sectionNadpis": "Nadobudnutie vlastnictva",
      "docId": "SKEZ1234",
      "score": 12.5,
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
| `docId` | Document ID for all `/doc/{docId}/*` endpoints |
| `recordId` | Internal law-level record identifier |
| `docType` | Document type (Zakon, Vyhlaska, etc.) |
| `declaredDate` | Date declared in the collection |
| `validFrom` / `validTo` | Validity period of the matched version (null validTo = currently valid) |
| `vyhlasene` | True if this is the as-published version |
| `lawNumber` / `lawYear` | Law number and year (use with `/law/{number}/{year}` endpoint) |
| `sectionOznacenie` | Designation of the best-matching section (e.g. "SS 1") |
| `legalDomains` | Legal domain classifications |
| `highlight` | Search hit highlights with `<mark>` tags |

## Document Types (typ filter)

- `Zakon` - Law
- `Vyhlaska` - Decree
- `Oznamenie` - Notice
- `Opatrenie` - Measure
- `Nariadenie vlady` - Government regulation
- `Ustavny zakon` - Constitutional law

## Examples

### Search Slovak Laws by Topic

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "obciansky zakonnik",
    "typ": "Zakon",
    "limit": 10
  }' | jq '.results[] | {docId, title, docNumber, lawNumber, lawYear}'
```

### Search Laws Valid at a Specific Date

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "dan z pridanej hodnoty",
    "validAt": "2025-06-01",
    "limit": 10
  }' | jq '.results[] | {docId, title, validFrom, validTo}'
```

### Search Recent Legislation by Declared Date

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "ochrana osobnych udajov",
    "issuedFrom": "2024-01-01",
    "limit": 10
  }' | jq '.results[] | {docId, title, declaredDate}'
```

### Sort by Date

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ?sort=date&order=desc" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "pracovne pravo",
    "limit": 10
  }' | jq '.results[] | {docId, title, declaredDate}'
```

### Find Law by Document Number

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{
    "docNumber": "311/2001 Z. z.",
    "limit": 1
  }' | jq '.results[] | {docId, title, docNumber}'
```

## Working with eZbierka Documents

```bash
DOC_ID="SKEZ1234"

# Get document metadata
cdx-sk -s "cdx-sk://doc/${DOC_ID}/meta" | jq '.'

# Get full text (defaults to currently valid version)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text"

# Get TOC
cdx-sk -s "cdx-sk://doc/${DOC_ID}/toc" | jq '.'

# Get versions
cdx-sk -s "cdx-sk://doc/${DOC_ID}/versions" | jq '.'
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
cdx-sk -s "cdx-sk://law/SK/40/1964/versions" | jq '.'

# Get law metadata
cdx-sk -s "cdx-sk://law/SK/40/1964/meta" | jq '.'
```

### Versioning (timecutId)

Laws have multiple versions (timecuts). Use `/versions` to list them:

```bash
cdx-sk -s "cdx-sk://doc/SKEZ1234/versions" | jq '.versions[] | {versionId, validFrom, validTo, vyhlasene}'
```

Each version has a `versionId` you can pass as `timecutId` to `/text` and `/toc`:

```bash
# Get text for a specific version
cdx-sk -s "cdx-sk://doc/SKEZ1234/text?timecutId=1964_40_2025-01-01"

# Get TOC for a specific version
cdx-sk -s "cdx-sk://doc/SKEZ1234/toc?timecutId=1964_40_2025-01-01" | jq '.'
```

Without `timecutId`, the API defaults to the currently valid version.

### Parts (Paragraphs)

Use `/parts` to list available paragraph IDs, then retrieve specific paragraphs:

```bash
# List parts
cdx-sk -s "cdx-sk://doc/SKEZ1234/parts" | jq '.parts[] | {id, oznacenie, nadpis}'

# Get text of specific paragraphs
cdx-sk -s "cdx-sk://doc/SKEZ1234/text?part=paragraf-1&part=paragraf-2"
```

Each part item includes `textUrl` (cdx-sk:// link to that section's text) and `attachmentUrl` (link to PDF page where the section starts).

### Find Related Documents

```bash
# Implementing regulations
cdx-sk -s "cdx-sk://doc/SKEZ1234/related?type=IMPLEMENTING&limit=10" | \
  jq '.results[] | {docId, title}'

# Laws that amend this law
cdx-sk -s "cdx-sk://doc/SKEZ1234/related?type=AMENDED_BY&limit=10" | \
  jq '.results[] | {docId, title}'

# Get counts of all relation types
cdx-sk -s "cdx-sk://doc/SKEZ1234/related/counts" | jq '.'
```

Relation types:
- `IMPLEMENTING` - Implementing regulations (vykonavacie predpisy)
- `AMENDS` - Laws this regulation amends
- `AMENDED_BY` - Laws that amend this regulation
- `REPEALS` - Laws this regulation repeals
- `REFERENCING_DECISION` - Court decisions referencing this law
- `REFERENCED_LAW` - Laws referenced by a court decision
