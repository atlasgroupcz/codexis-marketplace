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

**Versioned domain:** Laws have multiple timecuts (versions) with validFrom/validTo. Results are collapsed by law (one result per law, best-matching version). Use `/versions` to list all versions. Pass `timecutId` to `/text`, `/toc`, `/parts` for a specific version; without it the API defaults to the currently valid version.

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
      "highlight": { "sectionText": ["text with <mark>highlights</mark>"] },
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
| `lawNumber` / `lawYear` | Law number and year (use with `/law/SK/{number}/{year}`) |
| `sectionOznacenie` | Designation of the best-matching section (e.g. "SS 1") |
| `legalDomains` | Legal domain classifications |
| `highlight` | Search hit highlights with `<mark>` tags |

## Document Types (typ filter)

`Zakon`, `Vyhlaska`, `Oznamenie`, `Opatrenie`, `Nariadenie vlady`, `Ustavny zakon`

## Examples

```bash
# Search by topic with type filter
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{"query": "obciansky zakonnik", "typ": "Zakon", "limit": 10}' \
  | jq '.results[] | {docId, title, docNumber, lawNumber, lawYear}'

# Search laws valid at a specific date
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{"query": "dan z pridanej hodnoty", "validAt": "2025-06-01", "limit": 10}' \
  | jq '.results[] | {docId, title, validFrom, validTo}'

# Sort by date descending
cdx-sk -s -X POST "cdx-sk://search/SKEZ?sort=date&order=desc" \
  -H 'Content-Type: application/json' \
  -d '{"query": "pracovne pravo", "limit": 10}' \
  | jq '.results[] | {docId, title, declaredDate}'

# Find law by document number
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{"docNumber": "311/2001 Z. z.", "limit": 1}' \
  | jq '.results[] | {docId, title, docNumber}'
```

## Working with Documents

```bash
DOC_ID="SKEZ1234"

# Metadata (default: current version assets only; add ?includeAllAssets=true for all)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/meta" | jq '.'

# Full text (defaults to currently valid version)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text"

# Text of specific paragraphs
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?part=paragraf-1&part=paragraf-2"

# Text for a specific version
cdx-sk -s "cdx-sk://doc/${DOC_ID}/text?timecutId=1964_40_2025-01-01"

# TOC
cdx-sk -s "cdx-sk://doc/${DOC_ID}/toc" | jq '.'

# Versions (list all timecuts)
cdx-sk -s "cdx-sk://doc/${DOC_ID}/versions" | jq '.versions[] | {versionId, validFrom, validTo, vyhlasene}'
```

### Parts (Paragraphs)

List paragraph IDs with optional search and pagination (max limit: 500). Each part includes `textUrl` (cdx-sk:// link with timecutId) and `attachmentUrl` (PDF page link).

```bash
# List parts
cdx-sk -s "cdx-sk://doc/SKEZ1234/parts" | jq '.parts[] | {id, oznacenie, nadpis}'

# Search parts by id or designation (case-insensitive substring)
cdx-sk -s "cdx-sk://doc/SKEZ1234/parts?search=paragraf-23&offset=0&limit=50" | jq '.'

# Get text of a specific part
cdx-sk -s "cdx-sk://doc/SKEZ1234/text?part=paragraf-123"
```

### Direct Access by Law Number/Year

If you know the law number and year (e.g. 40/1964), skip search entirely:

```bash
cdx-sk -s "cdx-sk://law/SK/40/1964" | jq '{docId, title, docNumber}'
cdx-sk -s "cdx-sk://law/SK/40/1964/meta" | jq '.'
cdx-sk -s "cdx-sk://law/SK/40/1964/text"
cdx-sk -s "cdx-sk://law/SK/40/1964/toc" | jq '.'
cdx-sk -s "cdx-sk://law/SK/40/1964/versions" | jq '.'
cdx-sk -s "cdx-sk://law/SK/40/1964/parts?search=paragraf-23&offset=0&limit=50" | jq '.'
cdx-sk -s "cdx-sk://law/SK/40/1964/related?type=AMENDED_BY&limit=10" | jq '.'
cdx-sk -s "cdx-sk://law/SK/40/1964/related/counts" | jq '.'
```

### Related Documents

```bash
cdx-sk -s "cdx-sk://doc/SKEZ1234/related?type=IMPLEMENTING&limit=10" | jq '.results[] | {docId, title}'
cdx-sk -s "cdx-sk://doc/SKEZ1234/related?type=AMENDED_BY&limit=10" | jq '.results[] | {docId, title}'
cdx-sk -s "cdx-sk://doc/SKEZ1234/related/counts" | jq '.'
```

Relation types: `IMPLEMENTING` (implementing regulations), `AMENDS` (laws this regulation amends), `AMENDED_BY` (laws that amend this regulation), `REPEALS` (laws this regulation repeals), `REFERENCING_DECISION` (court decisions referencing this law), `REFERENCED_LAW` (laws referenced by a court decision).
