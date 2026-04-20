---
uuid: e8dcfc20-9582-42d6-81c8-d8c9aeec145a
name: cdx-sk
description: This skill should be invoked whenever user needs Slovak law or legal information — legislation from e-Zbierka, general court decisions, or supreme/constitutional court decisions.
version: 2.1.0
i18n:
  cs:
    displayName: "Slovenská legislativa"
    summary: "Slovenská legislativa (e-Zbierka) a rozhodnutí obecných, Nejvyššího a Ústavního soudu SR."
  en:
    displayName: "Slovak Legislation"
    summary: "Slovak legislation (e-Zbierka) plus general, Supreme and Constitutional Court decisions."
  sk:
    displayName: "Slovenská legislatíva"
    summary: "Slovenská legislatíva (e-Zbierka) a rozhodnutia všeobecných, Najvyššieho a Ústavného súdu SR."
---

# Slovak Legal Database (cdx-sk)

Slovak legal database providing structured access to three sources: e-Zbierka legislation (SKEZ), general court decisions (SKVS, ~4.6M decisions from district and regional courts), and supreme & constitutional court decisions (SKNUS, ~2.9K decisions from Najvyssi sud SR and Ustavny sud SR).

## Commands

### search
Search documents: `cdx-sk search <SOURCE> [OPTIONS]`

Use `cdx-sk search <SOURCE> --help` for available filters.

### get
Fetch a document resource: `cdx-sk get <cdx-sk://URL> [--dry-run]`

Use `cdx-sk get --help` for available URL patterns.

### schema
Print response schema for get endpoints: `cdx-sk schema <ENDPOINT> [SOURCE]`

Use `cdx-sk schema --help` for available endpoints.

## User-Facing Output Rules

All responses shown to the user **must** follow these formatting rules. The raw identifiers, codes, and enums from the API are for constructing CLI calls only — they must never leak into user-visible text.

### Link Format

**IMPORTANT:** All document links in user-facing output MUST use the `cdx-sk://` scheme. The system automatically resolves these to real URLs at render time. Never resolve URLs yourself — never read or use `$CDX_SK_API_URL` for link construction.

Every document reference MUST be a clickable attachment link — never plain text. How to build the link depends on the source:

- **SKEZ (legislation):** Use the `attachmentUrl` field from the `/parts` response directly — it already includes `#page=N`. If you haven't fetched `/parts`, get the filename from `/meta` assets and link without `#page`. When `attachmentUrl` is absent (PDF page mapping unavailable), fall back to a link without `#page`.
- **SKNUS/SKVS (court decisions):** Get the filename from `/meta` assets. Link to the attachment without `#page` — no page-level data exists for court decisions. Their `/parts` only provide `textUrl` for section text, not attachment URLs.

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

## Hard Rules

### Do Not Use includeAllAssets=true on /meta

The SKEZ `/meta` endpoint defaults to `includeAllAssets=false`, which returns only the current timecut's assets. This is correct for virtually all use cases. Do NOT pass `includeAllAssets=true` unless the user explicitly asks for all historical versions.

Example: Trestný zákonník has ~80 timecut versions, each with its own set of PDF assets. Passing `includeAllAssets=true` returns all of them — dozens of PDFs across all versions — which is never useful when answering a question about current law.

## Reference Files

- **`references/search-skez.md`** — Slovak legislation search (e-Zbierka): document types, validity filters, law number lookup
- **`references/search-skvs.md`** — Slovak general court decisions: court codes, decision forms, case file numbers
- **`references/search-sknus.md`** — Slovak supreme & constitutional court decisions: legal sentences, ECLI lookup
