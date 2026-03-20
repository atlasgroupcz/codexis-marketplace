---
name: cdx-at
description: This skill should be invoked whenever user needs Austrian law or legal information — federal law (Bundesrecht), case law (Judikatur), consolidated law history, state law (Landesrecht), or other publications (Sonstige) from the RIS system.
version: 1.0.0
---

# Austrian Legal Database API (cdx-at)

Austrian legal database (RIS — Rechtsinformationssystem) providing structured access to federal legislation, case law, consolidated law history, state law, and other publications via a REST API.

Use the `cdx-at` CLI for all requests. It accepts standard curl flags and `cdx-at://` URLs (for example `cdx-at://search/ATJD` or `cdx-at://doc/ATJD1234/text`).

## Data Sources

> **Note:** The source codes below (`ATBR`, `ATJD`, etc.) are internal API identifiers used to construct `cdx-at://` URLs. Never expose them in user-facing output — use human-readable names instead (see [User-Facing Output Rules](#user-facing-output-rules)).

| Code | Name | Description |
|------|------|-------------|
| `ATBR` | Bundesrecht (Federal Law) | BGBl documents — federal legislation as published in the Bundesgesetzblatt |
| `ATJD` | Judikatur (Case Law) | Court decisions from 7 courts: VfGH, VwGH, Justiz, BVwG, LVwG, Dok, Umse |
| `ATLR` | Landesrecht (State Law) | Consolidated state/provincial legislation |
| `ATSO` | Sonstige (Other Publications) | Directives, decrees, and other official publications |
| `ATHI` | History (Consolidated Federal Norms) | Historical consolidated federal law norms with amendment chains |

## User-Facing Output Rules

All responses shown to the user **must** follow these formatting rules. The raw identifiers, codes, and enums from the API are for constructing CLI calls only — they must never leak into user-visible text.

### Link Format

**IMPORTANT:** All document links in user-facing output MUST use the `cdx-at://` scheme. The system automatically resolves these to real URLs at render time. Never resolve URLs yourself — never read or use `$CDX_AT_API_URL` for link construction.

When citing documents, link to **attachment** URLs: `[Title](cdx-at://doc/{id}/attachment/{filename})`.
Get the filename from the `/meta` response (assets array).

Never present search, meta, text, or other API endpoints as clickable links — those are internal tool calls only.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `ATBR1234`, `ATJD5678`)
- Raw search prefixes (e.g., `ATBR`, `ATJD`, `ATHI`)
- Resolved HTTP URLs (e.g., `https://search.example.com/api/AT/judikatur/doc/...`)
- Environment variable names (e.g., `$CDX_AT_API_URL`)
- HTML tags (e.g., `<a href=...>`) — use markdown links only

### Human-Readable Source Names

When referring to data sources in prose, match the user's conversation language:

| Code (internal) | German Name | English Name |
|---|---|---|
| `ATBR` | Bundesrecht | Federal Legislation |
| `ATJD` | Judikatur | Case Law |
| `ATLR` | Landesrecht | State Law |
| `ATSO` | Sonstige Kundmachungen | Other Publications |
| `ATHI` | Konsolidiertes Bundesrecht (Historie) | Consolidated Federal Norms (History) |

### Document Titles

Use these fields as the link text:

- **ATBR:** `title` or `shortTitle` (e.g., "Bundesgesetz, mit dem das Strafgesetzbuch geandert wird")
- **ATJD:** `court` + case number from search results (e.g., `[VfGH, G 47/2024](cdx-at://doc/ATJD5678/attachment/content_1.pdf)`)
- **ATHI:** `shortTitle` or `abbreviation` + `articleParagraph` (e.g., "StGB § 165")
- **ATLR/ATSO:** `title` from search results

If title is unavailable, use `documentNumber` or a descriptive fallback — never the raw document ID.

### Examples

**Correct:**
```
[StGB § 165 — Betrügerischer Datenverarbeitungsmissbrauch](cdx-at://doc/ATHI1234/attachment/content_1.pdf)

[VfGH, G 47/2024](cdx-at://doc/ATJD5678/attachment/content_1.pdf)

[BGBl. I Nr. 58/2018](cdx-at://doc/ATBR9012/attachment/content_1.pdf)
```

**Incorrect:**
```
ATJD5678 — wrong, raw document ID
cdx-at://doc/ATJD5678/text — wrong, API endpoint as link
https://search.example.com/api/AT/judikatur/doc/ATJD5678 — wrong, resolved URL
```

## Core API Operations

### Search Documents

All search endpoints use POST with JSON body:

```bash
cdx-at -s -X POST "cdx-at://search/{ATBR|ATJD|ATLR|ATSO|ATHI}" \
  -H 'Content-Type: application/json' \
  -d '{"query": "search terms", "limit": 10}'
```

Supports fulltext search. Write German characters directly (umlauts handled automatically).

### Document Retrieval

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/doc/{docId}/meta` | GET | Document metadata (includes assets array with filenames) |
| `/doc/{docId}/text` | GET | Full document text (markdown) |
| `/doc/{docId}/attachment/{filename}` | GET | PDF/content file (use for user-facing links) |

### Resolve Display ID

```bash
cdx-at -s "cdx-at://resolve/{displayId}"   # auto-routes by prefix
```

## Document ID Conventions

IDs are internal — never show them to users (see [User-Facing Output Rules](#user-facing-output-rules)).

- **ATBR**: Federal legislation / BGBl documents
- **ATJD**: Court decisions (Judikatur)
- **ATLR**: State law (Landesrecht)
- **ATSO**: Other publications (Sonstige)
- **ATHI**: Consolidated federal norms (History)

Routing is automatic based on prefix. There is no cross-domain search.

## Working with Results

### Extract Fields with jq

```bash
# docId + title from search
cdx-at -s -X POST "cdx-at://search/ATJD" \
  -H 'Content-Type: application/json' \
  -d '{"query": "Grundrecht", "limit": 5}' \
  | jq '.results[] | {docId, title}'

# Attachment filename from /meta (needed for user-facing links)
cdx-at -s "cdx-at://doc/ATJD1234/meta" | jq '.assets[]'
```

## Quick Examples

### Search Austrian Case Law

```bash
cdx-at -s -X POST "cdx-at://search/ATJD" \
  -H 'Content-Type: application/json' \
  -d '{"query": "Meinungsfreiheit", "limit": 5}' \
  | jq '.results[] | {docId, title, court, decisionDate}'
```

### Search Federal Legislation

```bash
cdx-at -s -X POST "cdx-at://search/ATBR" \
  -H 'Content-Type: application/json' \
  -d '{"query": "Datenschutz", "limit": 5}' \
  | jq '.results[] | {docId, title, gazetteNumber}'
```

### Search Consolidated Law History

```bash
cdx-at -s -X POST "cdx-at://search/ATHI" \
  -H 'Content-Type: application/json' \
  -d '{"query": "StGB", "limit": 5}' \
  | jq '.results[] | {docId, abbreviation, shortTitle, articleParagraph}'
```

### Get Document Text

```bash
cdx-at -s "cdx-at://doc/ATJD1234/meta" | jq '.assets[]'
cdx-at -s "cdx-at://doc/ATJD1234/text"
```

## Best Practices

1. **Use specific source codes** — always search `ATBR`, `ATJD`, `ATLR`, `ATSO`, or `ATHI` directly. There is no cross-domain search endpoint.
2. **Use `cdx-at://` links** — always use `cdx-at://doc/{id}/attachment/{filename}` for user-facing links, never resolve URLs manually.
3. **Use jq for filtering** — process JSON results with jq rather than multiple API calls.
4. **Strip `<mark>` tags** — search highlights include `<mark>` tags; remove them before displaying titles.
5. **Get attachment filenames from /meta** — never guess filenames. Always fetch `/meta` first for the `assets` array.

## Reference Files

For detailed request/response schemas and filter options, consult:

- **`references/search-atjd.md`** — Austrian case law search (Judikatur): courts, decision types, ECLI, case numbers
- **`references/search-atbr.md`** — Austrian federal legislation search (Bundesrecht): gazette numbers, document types
- **`references/search-athi.md`** — Austrian consolidated law history: law abbreviations, amendments, effective dates
