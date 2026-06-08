---
uuid: 59dbdfcc-047f-41fd-97f0-fc958c85cf0c
name: cdx-cz-psp
description: This skill should be invoked whenever user needs Czech parliamentary information — legislative proposals (bills, amendments), parliamentary documents (reports, interpellations, EU docs) from the Czech Parliament (PSP) system.
version: 2.1.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "Poslanecká sněmovna ČR"
    summary: "Parlamentní tisky, interpelace, evropské dokumenty a sledování legislativních návrhů v PSP ČR."
  en:
    displayName: "Czech Parliament"
    summary: "Parliamentary papers, interpellations, EU documents and legislative bill tracking from the Czech Chamber of Deputies."
  sk:
    displayName: "Poslanecká snemovňa ČR"
    summary: "Parlamentné tlače, interpelácie, európske dokumenty a sledovanie legislatívnych návrhov v PSP ČR."
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

User-facing document links MUST use the `https://` URL that appears in tool output. The binary resolves the PDF attachment of each result to a real `https://…/attachment/…#page=N` link before you see it — that resolved link is the citable **source**; link to it: `[Title](https://…#page=N)`.

A `cdx-cz-psp://` link is NOT a user-facing link — it is an internal address you dereference with `cdx-cz-psp get cdx-cz-psp://…` to fetch more (`/meta`, `/text`, `/toc`). Never show a `cdx-cz-psp://` link to the user; if you need its content, fetch it and continue.

Never resolve URLs yourself and never read `$CODEXIS_PLUGIN_CZ_PSP_API_URL` for link construction. Strip `<mark>` tags from titles.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `CZPSPDOK1234`, `CZPSPPRE5678`)
- Raw search prefixes (e.g., `CZPSPDOK`, `CZPSPPRE`)
- Bare API URLs you construct yourself (e.g., `https://…/api/CZ/psp/preleg/doc/CZPSPPRE123`) — cite ONLY the `https://…/attachment/…` PDF link that appears in tool output
- Environment variable names (e.g., `$CODEXIS_PLUGIN_CZ_PSP_API_URL`)
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

Include press number and election period for context when helpful (e.g., `[Sněmovní tisk 123/0, 10. volební období](https://…/CZ/psp/preleg/doc/CZPSPPRE5678/attachment/content_1.pdf#page=2)`).

If title is unavailable, use press number or a descriptive fallback — never the raw document ID.

### Examples

**Correct:**
```
[Novela zákona o daních z příjmů (tisk 123)](https://…/CZ/psp/preleg/doc/CZPSPPRE1234/attachment/content_1.pdf#page=3)

[Písemná interpelace ve věci dopravní infrastruktury](https://…/CZ/psp/dokumenty/doc/CZPSPDOK5678/attachment/content_1.pdf)
```

**Incorrect:**
```
CZPSPPRE1234 — wrong, raw document ID
cdx-cz-psp://doc/CZPSPPRE1234/text — wrong, API endpoint as link
https://search.example.com/api/CZ/psp/preleg/doc/CZPSPPRE1234 — wrong, bare API doc URL (cite the /attachment/ PDF link from tool output instead)
```

## Hard Rules

### Always Link to Attachments

Every document reference in user-facing output MUST be a clickable attachment link. Never mention a document as plain text when you have the data to build a link.

- Both PSP sources (CZPSPPRE, CZPSPDOK) are page-native: search results include a ready-made `pageUrl` field — use it directly as the link target. The binary has already resolved it to a complete `https://…/attachment/…` URL with `#page=N` built in.
- When `pageUrl` is absent (the field is omitted from JSON when unavailable), get the filename from `/meta` assets and link to the resolved `https://…/attachment/…` URL without `#page` rather than omitting the link.

### Do Not Use includeAllAssets=true on /meta

By default, `/meta` returns only the primary content files (content_1.pdf, content_1.docx). Some documents — especially state budgets — have 50+ attachment files across multiple sub-prints.

Calling `/meta?includeAllAssets=true` returns the full list and can produce very large responses. Only use it when the user explicitly needs the complete file listing.

## Reference Files

- **`references/search-czpspdok.md`** — Czech parliamentary documents search: reports, interpellations, EU docs, committee assignments
- **`references/search-czpsppre.md`** — Czech legislative proposals search: bill types, legislative history, Sbírka zákonů publication, EUROVOC descriptors
