---
name: cdx-cz-psp
description: This skill should be invoked whenever user needs Czech parliamentary information — legislative proposals (bills, amendments), parliamentary documents (reports, interpellations, EU docs) from the Czech Parliament (PSP) system.
version: 1.0.0
---

# Czech Parliament Database (cdx-cz-psp)

Czech Parliament database (PSP — Poslanecká sněmovna Parlamentu ČR) providing structured access to two sources: parliamentary documents including reports, interpellations, and EU documents (CZPSPDOK), and legislative proposals including government bills, MP bills, senate bills, international treaties, and state budgets with full legislative history tracking (CZPSPPRE).

## Commands

### search
Search documents: `cdx-cz-psp search <SOURCE> [OPTIONS]`

Use `cdx-cz-psp search <SOURCE> --help` for available filters.

### get
Fetch a document resource: `cdx-cz-psp get <cdx-cz-psp://URL> [--dry-run]`

Use `cdx-cz-psp get --help` for available URL patterns.

### schema
Print response schema for get endpoints: `cdx-cz-psp schema <ENDPOINT> [SOURCE]`

Use `cdx-cz-psp schema --help` for available endpoints.

## User-Facing Output Rules

All responses shown to the user **must** follow these formatting rules. The raw identifiers, codes, and enums from the API are for constructing CLI calls only — they must never leak into user-visible text.

### Link Format

**IMPORTANT:** All document links in user-facing output MUST use the `cdx-cz-psp://` scheme. The system automatically resolves these to real URLs at render time. Never resolve URLs yourself — never read or use `$CDX_CZ_PSP_API_URL` for link construction.

When citing documents, link to **attachment** URLs: `[Title](cdx-cz-psp://doc/{id}/attachment/{filename})`.
Get the filename from the `/meta` response (assets array).

Never present search, meta, text, or other API endpoints as clickable links — those are internal tool calls only.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `CZPSPDOK1234`, `CZPSPPRE5678`)
- Raw search prefixes (e.g., `CZPSPDOK`, `CZPSPPRE`)
- Resolved HTTP URLs (e.g., `https://search.example.com/api/CZ/psp/dokumenty/doc/...`)
- Environment variable names (e.g., `$CDX_CZ_PSP_API_URL`)
- HTML tags (e.g., `<a href=...>`) — use markdown links only

### Human-Readable Source Names

When referring to data sources in prose, match the user's conversation language:

| Code (internal) | Czech Name | English Name |
|---|---|---|
| `CZPSPDOK` | Parlamentní dokumenty | Parliamentary Documents |
| `CZPSPPRE` | Legislativní návrhy | Legislative Proposals |

### Document Titles

Use these fields as the link text:

- **CZPSPDOK:** `title` or `fullTitle` from search results (e.g., "Písemná interpelace ve věci...")
- **CZPSPPRE:** `title` or `fullTitle` from search results (e.g., "Novela zákona o daních z příjmů")

Include press number and election period for context when helpful (e.g., `[Sněmovní tisk 123/0, 10. volební období](cdx-cz-psp://doc/CZPSPPRE5678/attachment/content_1.pdf)`).

If title is unavailable, use press number or a descriptive fallback — never the raw document ID.

### Examples

**Correct:**
```
[Novela zákona o daních z příjmů (tisk 123)](cdx-cz-psp://doc/CZPSPPRE1234/attachment/content_1.pdf)

[Písemná interpelace ve věci dopravní infrastruktury](cdx-cz-psp://doc/CZPSPDOK5678/attachment/content_1.pdf)
```

**Incorrect:**
```
CZPSPPRE1234 — wrong, raw document ID
cdx-cz-psp://doc/CZPSPPRE1234/text — wrong, API endpoint as link
https://search.example.com/api/CZ/psp/preleg/doc/CZPSPPRE1234 — wrong, resolved URL
```

## Hard Rules

### Always Link to Attachments

Every document reference in user-facing output MUST be a clickable attachment link. Never mention a document as plain text when you have the data to build a link.

- Get the filename from `/meta` assets.
- If `/meta` hasn't been fetched yet, fetch it to get the attachment filename before presenting the document to the user.

## Reference Files

- **`references/search-czpspdok.md`** — Czech parliamentary documents search: reports, interpellations, EU docs, committee assignments
- **`references/search-czpsppre.md`** — Czech legislative proposals search: bill types, legislative history, Sbírka zákonů publication, EUROVOC descriptors
