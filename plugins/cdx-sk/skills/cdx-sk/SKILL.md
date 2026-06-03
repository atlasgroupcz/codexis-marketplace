---
uuid: e8dcfc20-9582-42d6-81c8-d8c9aeec145a
name: cdx-sk
description: This skill should be invoked whenever user needs Slovak law or legal information — legislation from e-Zbierka, general court decisions, or supreme/constitutional court decisions.
version: 2.1.0
i18n:
  cs:
    displayName: "Slovenské právo (e-Zbierka a soudy SR)"
    summary: "Slovenská právní databáze — legislativa z e-Zbierky, ~4,6 mil. rozhodnutí všeobecných soudů a rozhodnutí Nejvyššího a Ústavního soudu SR."
  en:
    displayName: "Slovak Legal Database (e-Zbierka & courts)"
    summary: "Slovak legal database — e-Zbierka legislation, ~4.6M general court decisions and Supreme & Constitutional Court rulings."
  sk:
    displayName: "Slovenské právo (e-Zbierka a súdy SR)"
    summary: "Slovenská právna databáza — legislatíva z e-Zbierky, ~4,6 mil. rozhodnutí všeobecných súdov a rozhodnutia Najvyššieho a Ústavného súdu SR."
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

User-facing document links MUST use the `https://` URL that appears in tool output. The binary resolves PDF attachment links to real `https://.../attachment/...` URLs before you see them — that resolved link is the citable source.

A `cdx-sk://` link is NOT a user-facing link — it is an internal address you dereference with `cdx-sk get cdx-sk://...` to fetch more (`/meta`, `/text`, `/parts`, `/toc`). Never show a `cdx-sk://` link to the user; if you need its content, fetch it and continue.

Every document reference MUST be a clickable attachment link — never plain text. How to build the link depends on the source:

- **SKEZ (legislation):** Use the resolved `https://` attachment URL from `attachmentUrl`/`pageUrl` in the tool output; it may already include `#page=N`. If you haven't fetched `/parts`, get the filename from `/meta` assets and fetch/construct the internal `cdx-sk://.../attachment/...` URL only as a tool step, then cite the resolved `https://` URL from output.
- **SKNUS/SKVS (court decisions):** Get the filename from `/meta` assets. Link to the resolved attachment URL without `#page` — no page-level data exists for court decisions. Their `/parts` only provide `textUrl` for section text, not attachment URLs.

Never present search, meta, text, or other API endpoints as clickable links — those are internal tool calls only.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `SKEZ1234`, `SKVS5678`, `SKNUS9012`)
- Raw search prefixes (e.g., `SKEZ`, `SKVS`, `SKNUS`)
- Bare API URLs you construct yourself (e.g., `https://.../api/SK/ezbierka/doc/SKEZ123`) — cite ONLY the `https://.../attachment/...` PDF link that appears in tool output
- Environment variable names (e.g., `$CODEXIS_PLUGIN_SK_API_URL`)
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
- **SKVS/SKNUS:** `title` from search results, include court name inside the link text (e.g., `[Rozsudok — Okresny sud Bratislava I, sp. zn. 1C/123/2024](https://.../SK/vseobecne-sudy/doc/SKVS5678/attachment/content_1.pdf)`)

If title is unavailable, use `docNumber` or a descriptive fallback — never the raw document ID.

### Examples

**Correct:**
```
[ss 123 Obcianskeho zakonnika (40/1964 Zb.)](https://.../SK/ezbierka/doc/SKEZ1234/attachment/content_1.pdf#page=45)

[Rozsudok — Okresny sud Bratislava I, sp. zn. 1C/123/2024](https://.../SK/vseobecne-sudy/doc/SKVS5678/attachment/content_1.pdf)
```

**Incorrect:**
```
SKEZ1234 — wrong, raw document ID
cdx-sk://doc/SKEZ1234/text — wrong, API endpoint as link
cdx-sk://doc/SKEZ1234/attachment/content_1.pdf — wrong, internal address shown to user
https://search.example.com/api/SK/ezbierka/doc/SKEZ1234 — wrong, bare API doc URL (cite the /attachment/ PDF link from tool output instead)
```

## Hard Rules

### Do Not Use includeAllAssets=true on /meta

The SKEZ `/meta` endpoint defaults to `includeAllAssets=false`, which returns only the current timecut's assets. This is correct for virtually all use cases. Do NOT pass `includeAllAssets=true` unless the user explicitly asks for all historical versions.

Example: Trestný zákonník has ~80 timecut versions, each with its own set of PDF assets. Passing `includeAllAssets=true` returns all of them — dozens of PDFs across all versions — which is never useful when answering a question about current law.

## Reference Files

- **`references/search-skez.md`** — Slovak legislation search (e-Zbierka): document types, validity filters, law number lookup
- **`references/search-skvs.md`** — Slovak general court decisions: court codes, decision forms, case file numbers
- **`references/search-sknus.md`** — Slovak supreme & constitutional court decisions: legal sentences, ECLI lookup
