---
name: cdx-sk
description: This skill should be invoked whenever user needs Slovak law or legal information — legislation from e-Zbierka, general court decisions, or supreme/constitutional court decisions.
version: 1.0.0
---

# Slovak Legal Database API (cdx-sk)

Slovak legal database providing structured access to legislation (e-Zbierka), general court decisions, and supreme & constitutional court decisions via a REST API.

Use the `cdx-sk` CLI for all requests. It accepts standard curl flags and `cdx-sk://` URLs (for example `cdx-sk://search/SKEZ` or `cdx-sk://doc/SKEZ1234/text`).

## Data Sources

> **Note:** The source codes below (`SKEZ`, `SKVS`, `SKNUS`) are internal API identifiers used to construct `cdx-sk://` URLs. Never expose them in user-facing output — use human-readable names instead (see [User-Facing Output Rules](#user-facing-output-rules)).

| Code | Name | Description | Has Versions |
|------|------|-------------|--------------|
| `SKEZ` | e-Zbierka (Slovak Legislation) | Laws, decrees, regulations from Zbierka zakonov SR | Yes (timecuts) |
| `SKVS` | General Courts | Decisions from okresne, krajske and other general courts (~4.6M) | No (single version) |
| `SKNUS` | Supreme & Constitutional Courts | Decisions from Najvyssi sud SR and Ustavny sud SR (~2.9K) | No (single version) |

## User-Facing Output Rules

All responses shown to the user **must** follow these formatting rules. The raw identifiers, codes, and enums from the API are for constructing CLI calls only — they must never leak into user-visible text.

### Link Format

**IMPORTANT:** All document links in user-facing output MUST use the `cdx-sk://` scheme. The system automatically resolves these to real URLs at render time. Never resolve URLs yourself — never read or use `$CDX_SK_API_URL` for link construction.

When citing documents, link to **attachment** URLs: `[Title](cdx-sk://doc/{id}/attachment/{filename}#page=N)`.
Get the filename from the `/meta` response (assets array) and the page from the `/parts` response.

Never present search, meta, text, or other API endpoints as clickable links — those are internal tool calls only.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `SKEZ1234`, `SKVS5678`, `SKNUS9012`)
- Raw search prefixes (e.g., `SKEZ`, `SKVS`, `SKNUS`)
- Resolved HTTP URLs (e.g., `https://search.example.com/api/SK/ezbierka/doc/...`)
- Environment variable names (e.g., `$CDX_SK_API_URL`)
- HTML tags (e.g., `<a href=...>`) — use markdown links only

### Human-Readable Source Names

When referring to data sources in prose, match the user's conversation language:

| Code (internal) | Slovak Name | English Name |
|---|---|---|
| `SKEZ` | Zbierka zakonov SR | Slovak Legislation |
| `SKVS` | Vseobecne sudy SR | Slovak General Courts |
| `SKNUS` | Najvyssi a Ustavny sud SR | Slovak Supreme & Constitutional Courts |

### Document Titles

Use these fields as the link text:

- **SKEZ:** `title` from search results (e.g., "Obciansky zakonnik") — strip `<mark>` tags
- **SKVS/SKNUS:** `title` from search results, include court name inside the link text (e.g., `[Rozsudok — Okresny sud Bratislava I, sp. zn. 1C/123/2024](cdx-sk://doc/SKVS5678/attachment/content_1.pdf)`)

If title is unavailable, use `docNumber` or a descriptive fallback — never the raw document ID.

### Examples

**Correct:**
```
[ss 123 Obcianskeho zakonnika (40/1964 Zb.)](cdx-sk://doc/SKEZ1234/attachment/content_1.pdf#page=45)

[Rozsudok — Okresny sud Bratislava I, sp. zn. 1C/123/2024](cdx-sk://doc/SKVS5678/attachment/content_1.pdf)
```

**Incorrect:**
```
SKEZ1234 — wrong, raw document ID
cdx-sk://doc/SKEZ1234/text — wrong, API endpoint as link
https://search.example.com/api/SK/ezbierka/doc/SKEZ1234 — wrong, resolved URL
```

## Core API Operations

### Search Documents

All search endpoints use POST with JSON body:

```bash
cdx-sk -s -X POST "cdx-sk://search/{SKEZ|SKVS|SKNUS}" \
  -H 'Content-Type: application/json' \
  -d '{"query": "search terms", "limit": 10}'
```

Supports fulltext, wildcards (`zmluv*`), phrases (`"najem bytu"`). Write Slovak characters directly.

### Document Retrieval

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/doc/{docId}/meta` | GET | Document metadata (includes assets array with filenames) |
| `/doc/{docId}/text` | GET | Full document text |
| `/doc/{docId}/attachment/{filename}` | GET | PDF/content file (use for user-facing links) |
| `/doc/{docId}/parts` | GET | Available sections/paragraphs with page references |
| `/doc/{docId}/versions` | GET | Version list (SKEZ: multiple timecuts; SKVS/SKNUS: single version) |
| `/doc/{docId}/related` | GET | Related documents |
| `/doc/{docId}/related/counts` | GET | Relation type counts |
| `/doc/{docId}/toc` | GET | Table of contents (SKEZ only) |

### Direct Law Access (SKEZ Only)

When the law reference is known (e.g., `40/1964 Zb.`), skip search:

```bash
cdx-sk -s "cdx-sk://law/SK/{number}/{year}"          # resolve
cdx-sk -s "cdx-sk://law/SK/{number}/{year}/text"      # full text
cdx-sk -s "cdx-sk://law/SK/{number}/{year}/meta"      # metadata
cdx-sk -s "cdx-sk://law/SK/{number}/{year}/toc"       # table of contents
cdx-sk -s "cdx-sk://law/SK/{number}/{year}/versions"  # version list
cdx-sk -s "cdx-sk://law/SK/{number}/{year}/parts"     # sections/paragraphs
cdx-sk -s "cdx-sk://law/SK/{number}/{year}/related?type=T"       # related documents
cdx-sk -s "cdx-sk://law/SK/{number}/{year}/related/counts"       # relation type counts
```

`number` is the part before `/` (e.g., `40`), `year` after (e.g., `1964`). Defaults to the currently valid version.

**Parts search and pagination (SKEZ):**
```bash
cdx-sk -s "cdx-sk://law/SK/40/1964/parts?search=paragraf-23"           # filter by ID or designation
cdx-sk -s "cdx-sk://law/SK/40/1964/parts?offset=0&limit=50"            # paginate large laws
cdx-sk -s "cdx-sk://doc/SKEZ1234/parts?search=paragraf-23&limit=10"    # also works with docId
```

**Meta asset trimming (SKEZ):** By default, `/meta` returns only assets from the current version. Use `?includeAllAssets=true` to get all historical assets:
```bash
cdx-sk -s "cdx-sk://law/SK/40/1964/meta"                          # current version assets only
cdx-sk -s "cdx-sk://law/SK/40/1964/meta?includeAllAssets=true"    # all historical assets
```

### Resolve Display ID

```bash
cdx-sk -s "cdx-sk://resolve/{displayId}"   # auto-routes by prefix
```

## Document ID Conventions

IDs are internal — never show them to users (see [User-Facing Output Rules](#user-facing-output-rules)).

- **SKEZ**: e-Zbierka legislation (e.g., SKEZ1234)
- **SKVS**: General court decisions (e.g., SKVS5678)
- **SKNUS**: Supreme/constitutional court decisions (e.g., SKNUS9012)

Routing is automatic based on prefix. There is no cross-domain search.

## Working with Results

### Extract Fields with jq

```bash
# docId + title from search
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{"query": "obciansky zakonnik", "limit": 5}' \
  | jq '.results[] | {docId, title}'

# Attachment filename from /meta (needed for user-facing links)
cdx-sk -s "cdx-sk://doc/SKEZ1234/meta" | jq '.assets[]'

# Page numbers from /parts (for #page=N in links)
cdx-sk -s "cdx-sk://doc/SKEZ1234/parts" | jq '.parts[] | {id, oznacenie, startPage}'
```

### Follow-Up Patterns

```bash
# List legislation versions (SKEZ), pick specific one via timecutId
cdx-sk -s "cdx-sk://doc/SKEZ1234/versions" \
  | jq '.versions[] | {versionId, validFrom, validTo}'
cdx-sk -s "cdx-sk://doc/SKEZ1234/text?timecutId=1964_40_2020-01-01"

# List parts with page refs, then fetch specific paragraphs
cdx-sk -s "cdx-sk://doc/SKEZ1234/parts" \
  | jq '.parts[] | {id, oznacenie, nadpis, startPage}'
cdx-sk -s "cdx-sk://doc/SKEZ1234/text?part=paragraf-123&part=paragraf-124"
```

## Quick Examples

### Search Slovak Legislation

```bash
cdx-sk -s -X POST "cdx-sk://search/SKEZ" \
  -H 'Content-Type: application/json' \
  -d '{"query": "dan z prijmov", "validAt": "2026-01-01", "limit": 5}' \
  | jq '.results[] | {docId, title, docNumber}'
```

> Present results as attachment links: `[{title} ({docNumber})](cdx-sk://doc/{docId}/attachment/{filename})` — never raw JSON, IDs, or resolved HTTP URLs.

### Get a Court Decision PDF

```bash
# Search, then get attachment filename from /meta, then build link
cdx-sk -s -X POST "cdx-sk://search/SKVS" \
  -H 'Content-Type: application/json' \
  -d '{"query": "nahrada skody", "decisionForm": "Rozsudok", "limit": 3}' \
  | jq '.results[0] | {docId, title}'
cdx-sk -s "cdx-sk://doc/SKVS5678/meta" | jq '.assets[]'
# Link: [title](cdx-sk://doc/SKVS5678/attachment/content_1.pdf)
```

### Direct Law Access by Number/Year

```bash
cdx-sk -s "cdx-sk://law/SK/40/1964" | jq '{docId, title, docNumber}'
cdx-sk -s "cdx-sk://law/SK/40/1964/text"
```

### Find Related Documents

```bash
cdx-sk -s "cdx-sk://doc/SKEZ1234/related/counts" | jq '.'
cdx-sk -s "cdx-sk://doc/SKEZ1234/related?type=IMPLEMENTING&limit=10" \
  | jq '.results[] | {docId, title}'
```

## Workflow Recipes

### Law → Paragraph → Link (SKEZ)

When the user asks about a specific paragraph of a known law:

```bash
# 1. Search parts directly (no need to resolve docId first)
cdx-sk -s "cdx-sk://law/SK/40/1964/parts?search=paragraf-23" \
  | jq '.parts[] | {id, oznacenie, startPage, attachmentUrl}'

# 2. Use attachmentUrl directly from the response for the user-facing link
# Result: [§ 23 Obcianskeho zakonnika](cdx-sk://doc/SKEZ1234/attachment/content_1.pdf#page=45)
```

No resolve step needed — law-level endpoints handle it automatically.

### Find Related Laws (SKEZ)

```bash
# 1. Check what relation types exist
cdx-sk -s "cdx-sk://law/SK/40/1964/related/counts" | jq '.'

# 2. Fetch specific relation type
cdx-sk -s "cdx-sk://law/SK/40/1964/related?type=IMPLEMENTING&limit=10" \
  | jq '.results[] | {docId, title}'
```

## Best Practices

1. **Prefer `/parts?search=X` over `/meta`** — when you need a specific paragraph link, search parts directly instead of downloading full metadata.
2. **Use `attachmentUrl` from `/parts`** — never construct attachment URLs manually. The `/parts` response includes ready-to-use `attachmentUrl` for each part.
3. **Use law-level endpoints** — use `cdx-sk://law/SK/{number}/{year}/...` for all ezbierka operations when the law reference is known. No need to resolve docId first.
4. **Never mix docId and filenames** — never combine a docId from one API response with attachment filenames from another.
5. **Use `#page=N` from `/parts`** — get `startPage` from the parts response for page-level attachment links.
6. **Use `validAt` for legislation** — filter by `validAt: "YYYY-MM-DD"` to get the version valid at a specific date.
7. **Use specific source codes** — always search `SKEZ`, `SKVS`, or `SKNUS` directly. There is no cross-domain search endpoint.
8. **Use jq for filtering** — process JSON results with jq rather than multiple API calls.
9. **Strip `<mark>` tags** — search highlights include `<mark>` tags; remove them before displaying titles.
10. **Use `cdx-sk://` links** — always use `cdx-sk://doc/{id}/attachment/{filename}` for user-facing links, never resolve URLs manually.

## Reference Files

For detailed request/response schemas, filter options, and worked examples, consult:

- **`references/search-skez.md`** — Slovak legislation search (e-Zbierka): document types, validity filters, law number lookup
- **`references/search-skvs.md`** — Slovak general court decisions: court codes, decision forms, case file numbers
- **`references/search-sknus.md`** — Slovak supreme & constitutional court decisions: legal sentences, ECLI lookup
