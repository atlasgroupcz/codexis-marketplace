---
name: cdx-at
description: This skill should be invoked whenever user needs Austrian law or legal information — federal law (Bundesrecht), case law (Judikatur), consolidated law history, state law (Landesrecht), or other publications (Sonstige) from the RIS system.
version: 2.1.0
---

# Austrian Legal Database (cdx-at)

Austrian legal database (RIS — Rechtsinformationssystem) providing structured access to five sources: federal legislation published in the Bundesgesetzblatt (ATBR), court decisions from 7 courts including VfGH and VwGH (ATJD), consolidated state/provincial legislation (ATLR), directives and other official publications (ATSO), and historical consolidated federal law norms with amendment chains (ATHI).

## Commands

### search
Search documents: `cdx-at search <SOURCE> [OPTIONS]`

Use `cdx-at search <SOURCE> --help` for available filters.

### get
Fetch a document resource: `cdx-at get <cdx-at://URL> [--dry-run]`

Use `cdx-at get --help` for available URL patterns.

### schema
Print response schema for get endpoints: `cdx-at schema <ENDPOINT> [SOURCE]`

Use `cdx-at schema --help` for available endpoints.

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

## Hard Rules

### Always Link to Attachments

Every document reference in user-facing output MUST be a clickable attachment link. Never mention a document as plain text when you have the data to build a link.

- Get the filename from `/meta` assets. No page-level data is available in Austrian domains, so link without `#page`.
- If `/meta` hasn't been fetched yet, fetch it to get the attachment filename before presenting the document to the user.

## Reference Files

- **`references/search-atjd.md`** — Austrian case law search (Judikatur): courts, decision types, ECLI, case numbers
- **`references/search-atbr.md`** — Austrian federal legislation search (Bundesrecht): gazette numbers, document types
- **`references/search-athi.md`** — Austrian consolidated law history: law abbreviations, amendments, effective dates
