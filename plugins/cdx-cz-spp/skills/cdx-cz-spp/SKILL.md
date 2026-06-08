---
uuid: 6da16057-ae63-42ca-a1bf-0541a80da14a
name: cdx-cz-spp
description: This skill should be invoked whenever user needs Czech municipal regulations — ordinances, decrees, and other local legal acts from sbirkapp.gov.cz (Sbírka právních předpisů).
version: 2.1.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "Obecní předpisy ČR"
    summary: "Obecně závazné vyhlášky, nařízení a místní právní akty obcí a krajů ze sbirkapp.gov.cz."
  en:
    displayName: "Czech Municipal Regulations"
    summary: "Generally binding ordinances, decrees, and local legal acts from Czech municipalities (sbirkapp.gov.cz)."
  sk:
    displayName: "Obecné predpisy ČR"
    summary: "Všeobecne záväzné vyhlášky, nariadenia a miestne právne akty českých obcí a krajov (sbirkapp.gov.cz)."
---

# Czech Municipal Regulations (cdx-cz-spp)

Czech municipal regulations database providing structured access to ordinances (obecně závazné vyhlášky), regulations (nařízení), and other local legal acts published in the official collection at sbirkapp.gov.cz. All documents are from a single source: Sbírka právních předpisů (CZSB).

## Commands

### search
Search documents: `cdx-cz-spp search <SOURCE> [OPTIONS]`

Use `cdx-cz-spp search <SOURCE> --help` for available filters.

### get
Fetch a document resource: `cdx-cz-spp get <cdx-cz-spp://URL> [--dry-run]`

Use `cdx-cz-spp get --help` for available URL patterns.

### schema
Print response schema for get endpoints: `cdx-cz-spp schema <ENDPOINT> [SOURCE]`

Use `cdx-cz-spp schema --help` for available endpoints.

## User-Facing Output Rules

All responses shown to the user **must** follow these formatting rules. The raw identifiers, codes, and enums from the API are for constructing CLI calls only — they must never leak into user-visible text.

### Link Format

User-facing document links MUST use the `https://` URL that appears in tool output. The binary resolves the PDF attachment of each result to a real `https://…/attachment/…#page=N` link before you see it — that resolved link is the citable **source**; link to it: `[Title](https://…#page=N)`.

A `cdx-cz-spp://` link is NOT a user-facing link — it is an internal address you dereference with `cdx-cz-spp get cdx-cz-spp://…` to fetch more (`/meta`, `/text`, `/toc`). Never show a `cdx-cz-spp://` link to the user; if you need its content, fetch it and continue.

Never resolve URLs yourself and never read `$CODEXIS_PLUGIN_CZ_SPP_API_URL` for link construction. Strip `<mark>` tags from titles.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `CZSB1234`)
- Raw search prefix (`CZSB`)
- Bare API URLs you construct yourself (e.g., `https://…/api/CZ/sbirkapp/doc/CZSB123`) — cite ONLY the `https://…/attachment/…` PDF link that appears in tool output
- Environment variable names (e.g., `$CODEXIS_PLUGIN_CZ_SPP_API_URL`)
- HTML tags (e.g., `<a href=...>`) — use markdown links only

### Human-Readable Source Names

When referring to the data source in prose, match the user's conversation language:

| Code (internal) | Czech Name | English Name |
|---|---|---|
| `CZSB` | Sbírka právních předpisů | Czech Municipal Regulations |

### Document Titles

Use the `nazev` field as the link text, combined with publisher and document number for context:

- `[OZV o místním poplatku — Statutární město Brno, č. 8/2025](https://…/CZ/sbirkapp/doc/CZSB123/attachment/content_1.pdf)`

If nazev is unavailable, use `cisloPredpisu` and `publikujici` as fallback — never the raw document ID.

### Examples

**Correct:**
```
[OZV o místním poplatku za obecní systém odpadového hospodářství — Statutární město Brno, č. 8/2025](https://…/CZ/sbirkapp/doc/CZSB123/attachment/content_1.pdf#page=3)
```

**Incorrect:**
```
CZSB123 — wrong, raw document ID
cdx-cz-spp://doc/CZSB123/text — wrong, API endpoint as link
https://search.example.com/api/CZ/sbirkapp/doc/CZSB123 — wrong, bare API doc URL (cite the /attachment/ PDF link from tool output instead)
```

## Hard Rules

### Always Link to Attachments

Every document reference in user-facing output MUST be a clickable attachment link. Never mention a document as plain text when you have the data to build a link.

- Search results include a ready-made `pageUrl` field — use it directly as the link target. The binary has already resolved it to a complete `https://…/attachment/…` URL with `#page=N` built in.
- When `pageUrl` is absent (the field is omitted from JSON when unavailable), get the filename from `/meta` assets and link without `#page` rather than omitting the link.

## Reference Files

- **`references/search-czsb.md`** — Czech municipal regulations search: document types, date filters, publisher lookup, sorting
