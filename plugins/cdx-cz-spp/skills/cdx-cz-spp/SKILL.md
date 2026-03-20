---
name: cdx-cz-spp
description: This skill should be invoked whenever user needs Czech municipal regulations — ordinances, decrees, and other local legal acts from sbirkapp.gov.cz (Sbírka právních předpisů).
version: 1.0.0
---

# Czech Municipal Regulations API (cdx-cz-spp)

Czech municipal regulations database providing structured access to ordinances (obecně závazné vyhlášky), regulations (nařízení), and other local legal acts published in the official collection at sbirkapp.gov.cz.

Use the `cdx-cz-spp` CLI for all requests. It accepts standard curl flags and `cdx-cz-spp://` URLs (for example `cdx-cz-spp://search/CZSB` or `cdx-cz-spp://doc/CZSB123/text`).

## Data Source

> **Note:** The source code `CZSB` is an internal API identifier used to construct `cdx-cz-spp://` URLs. Never expose it in user-facing output — use human-readable names instead (see [User-Facing Output Rules](#user-facing-output-rules)).

| Code | Name | Description | Has Versions |
|------|------|-------------|--------------|
| `CZSB` | Sbírka právních předpisů | Municipal ordinances, regulations, constitutional court rulings, and other acts from Czech municipalities and regions | Yes (numbered versions) |

### Document Categories

| hlavniTyp | Name | Types |
|-----------|------|-------|
| `pp` | Právní předpisy (Legal regulations) | Obecně závazná vyhláška, Nařízení |
| `oa` | Ostatní akty (Other acts) | Nález Ústavního soudu, Rozhodnutí o pozastavení účinnosti, Smlouva, Stav nebezpečí |

## User-Facing Output Rules

All responses shown to the user **must** follow these formatting rules. The raw identifiers, codes, and enums from the API are for constructing CLI calls only — they must never leak into user-visible text.

### Link Format

**IMPORTANT:** All document links in user-facing output MUST use the `cdx-cz-spp://` scheme. The system automatically resolves these to real URLs at render time. Never resolve URLs yourself — never read or use `$CDX_CZ_SPP_API_URL` for link construction.

When citing documents, link to **attachment** URLs: `[Title](cdx-cz-spp://doc/{id}/attachment/{filename}#page=N)`.
Get the filename from the `/meta` response (assets array) and the page from search results (`pageNumber`) or `/parts`.

Never present search, meta, text, or other API endpoints as clickable links — those are internal tool calls only.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `CZSB1234`)
- Raw search prefix (`CZSB`)
- Resolved HTTP URLs (e.g., `https://search.example.com/api/CZ/sbirkapp/doc/...`)
- Environment variable names (e.g., `$CDX_CZ_SPP_API_URL`)
- HTML tags (e.g., `<a href=...>`) — use markdown links only

### Human-Readable Source Names

When referring to the data source in prose, match the user's conversation language:

| Code (internal) | Czech Name | English Name |
|---|---|---|
| `CZSB` | Sbírka právních předpisů | Czech Municipal Regulations |

### Document Titles

Use the `nazev` field as the link text, combined with publisher and document number for context:

- `[OZV o místním poplatku — Statutární město Brno, č. 8/2025](cdx-cz-spp://doc/CZSB123/attachment/content_1.pdf)`

If nazev is unavailable, use `cisloPredpisu` and `publikujici` as fallback — never the raw document ID.

### Examples

**Correct:**
```
[OZV o místním poplatku za obecní systém odpadového hospodářství — Statutární město Brno, č. 8/2025](cdx-cz-spp://doc/CZSB123/attachment/content_1.pdf)
```

**Incorrect:**
```
CZSB123 — wrong, raw document ID
cdx-cz-spp://doc/CZSB123/text — wrong, API endpoint as link
https://search.example.com/api/CZ/sbirkapp/doc/CZSB123 — wrong, resolved URL
```

## Core API Operations

### Search Documents

```bash
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB" \
  -H 'Content-Type: application/json' \
  -d '{"query": "search terms", "limit": 10}'
```

Supports fulltext, Czech characters. Query params `?sort=relevance|title|date&order=asc|desc` for sorting.

### Document Retrieval

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/doc/{docId}/meta` | GET | Document metadata (includes assets array with filenames) |
| `/doc/{docId}/text` | GET | Full document text (supports `?page=N`, `?part=ID`, `?file=F`, `?version=N`) |
| `/doc/{docId}/attachment/{filename}` | GET | PDF/content file (use for user-facing links) |
| `/doc/{docId}/parts` | GET | Available sections with headings and levels |
| `/doc/{docId}/versions` | GET | Version list (numbered versions with their own assets) |
| `/doc/{docId}/related` | GET | Related documents (supports `?type=T&limit=N`) |
| `/doc/{docId}/related/counts` | GET | Relation type counts |
| `/doc/{docId}/toc` | GET | Table of contents |

### Resolve Display ID

```bash
cdx-cz-spp -s "cdx-cz-spp://resolve/{displayId}"   # auto-routes by prefix
```

## Working with Results

### Extract Fields with jq

```bash
# docId + title from search
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB" \
  -H 'Content-Type: application/json' \
  -d '{"query": "odpad", "limit": 5}' \
  | jq '.results[] | {docId, nazev, publikujici, cisloPredpisu}'

# Attachment filename from /meta (needed for user-facing links)
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/meta" | jq '.assets[]'

# Sections from /parts
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/parts" | jq '.parts[] | {id, heading, level}'
```

### Follow-Up Patterns

```bash
# List document versions, pick specific one
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/versions" \
  | jq '.versions[] | {versionNum, nazev, assets}'
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/text?version=1"

# List parts, then fetch specific section text
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/parts" \
  | jq '.parts[] | {id, heading, level}'
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/text?part=section-1"
```

## Quick Examples

### Search Municipal Regulations by Topic

```bash
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB" \
  -H 'Content-Type: application/json' \
  -d '{"query": "odpadove hospodarstvi", "hlavniTyp": "pp", "limit": 5}' \
  | jq '.results[] | {docId, nazev, publikujici, cisloPredpisu}'
```

> Present results as attachment links: `[{nazev} — {publikujici}, č. {cisloPredpisu}](cdx-cz-spp://doc/{docId}/attachment/{sourceFile}.pdf)` — never raw JSON, IDs, or resolved HTTP URLs.

### Find Regulations by Municipality

```bash
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB" \
  -H 'Content-Type: application/json' \
  -d '{"publikujici": "Statutarni mesto Brno", "platnost": "Platne", "limit": 10}' \
  | jq '.results[] | {docId, nazev, cisloPredpisu, datumVydani}'
```

### Find Related Documents

```bash
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/related/counts" | jq '.'
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/related?type=AMENDED_BY&limit=10" \
  | jq '.results[] | {sourceId, nazev, cisloPredpisu}'
```

## Workflow Recipes

### Search → Read → Cite

When the user asks about a municipal regulation:

```bash
# 1. Search
cdx-cz-spp -s -X POST "cdx-cz-spp://search/CZSB" \
  -H 'Content-Type: application/json' \
  -d '{"query": "mistni poplatek ze psu", "hlavniTyp": "pp", "limit": 5}' \
  | jq '.results[] | {docId, nazev, publikujici, cisloPredpisu, pageUrl}'

# 2. Get text of the matching page
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/text?page=0"

# 3. Get asset filename for link
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/meta" | jq '.assets[]'

# 4. Present: [OZV o místním poplatku ze psů — Město X, č. 1/2025](cdx-cz-spp://doc/CZSB123/attachment/content_1.pdf)
```

### Track Amendment History

```bash
# 1. Check what relation types exist
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/related/counts" | jq '.'

# 2. Fetch amendments
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/related?type=AMENDED_BY&limit=10" \
  | jq '.results[] | {sourceId, nazev}'

# 3. Compare versions
cdx-cz-spp -s "cdx-cz-spp://doc/CZSB123/versions" \
  | jq '.versions[] | {versionNum, nazev}'
```

## Best Practices

1. **Use `pageUrl` from search results** — each result is a single page match with a ready-to-use `pageUrl` link. The same regulation may appear multiple times (once per matching page).
2. **Use `jq` for filtering** — process JSON results with jq rather than multiple API calls.
3. **Strip `<mark>` tags** — search highlights include `<mark>` tags; remove them before displaying titles.
4. **Use `cdx-cz-spp://` links** — always use the custom scheme for user-facing links, never resolve URLs manually.
5. **Filter by `hlavniTyp`** — use `pp` for regulations or `oa` for other acts to narrow results.
6. **Use `platnost` for validity** — filter by `"Platne"` to find currently valid regulations.
7. **Combine publisher + topic** — use `publikujici` with `query` to find specific municipality's regulations on a topic.
8. **Check versions** — municipal regulations are often amended; use `/versions` to list all versions before citing.

## Reference Files

For detailed request/response schemas, all filter options, document types, and worked examples, consult:

- **`references/search-czsb.md`** — Czech municipal regulations search: document types, date filters, publisher lookup, sorting
