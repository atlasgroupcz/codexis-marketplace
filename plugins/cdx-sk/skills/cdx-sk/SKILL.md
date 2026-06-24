---
uuid: e8dcfc20-9582-42d6-81c8-d8c9aeec145a
name: cdx-sk
icon: icon.svg
description: This skill should be invoked whenever user needs Slovak law or legal information — legislation from e-Zbierka, general court decisions, or supreme/constitutional court decisions.
version: 2.3.0
jurisdictions: [SK]
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

User-facing document links MUST use the `https://` URL that appears in tool output. The binary resolves the PDF attachment of each result to a real `https://…/attachment/…#page=N` link before you see it — that resolved link is the citable **source**; link to it: `[Title](https://…#page=N)`.

A `cdx-sk://` link is NOT a user-facing link — it is an internal address you dereference with `cdx-sk get cdx-sk://…` to fetch more (`/meta`, `/text`, `/toc`, `/parts`). Never show a `cdx-sk://` link to the user; if you need its content, fetch it and continue.

Never resolve URLs yourself and never read `$CODEXIS_PLUGIN_SK_API_URL` for link construction. Strip `<mark>` tags from titles.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `SKEZ1234`, `SKVS5678`, `SKNUS9012`)
- Raw search prefixes (e.g., `SKEZ`, `SKVS`, `SKNUS`)
- Bare API URLs you construct yourself (e.g., `https://…/api/SK/ezbierka/doc/SKEZ123`) — cite ONLY the `https://…/attachment/…` PDF link that appears in tool output
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

- **SKEZ (legislation):** the link text MUST be the compact law citation, not the bare title: `ČÍSLO NÁZOV` from `docNumber` + `title` (e.g. `[311/2001 Z. z. Zákonník práce](https://…/attachment/content_1.pdf#page=45)`). The collection suffix comes from `docNumber` as the data returns it — never hardcode it: pre-1993 laws are `Zb.`, post-1993 `Z. z.` (e.g. `40/1964 Zb. Občiansky zákonník`). When citing a specific version, append `, v znení účinnom od DD.MM.RRRR` (from the result's `validFrom`). The API's resolved source titles already use this exact format — where one is shown, reuse it verbatim. Never invent a missing segment: no number/year → `title` alone; no `validFrom` → drop only the version suffix.
- **SKVS/SKNUS (court decisions):** the link text MUST be a compact legal citation, not the generic title: `SÚD - SP. ZN. - DD.MM.RRRR` (e.g. `[OS Bratislava I - 1C/123/2024 - 15.03.2024](https://…/attachment/content_1.pdf#page=1)`). Map the court to its abbreviation per the tables in `references/search-skvs.md` / `references/search-sknus.md`; the reference is the case file number (else `ecli`), and append ` - DD.MM.RRRR` (decision date) when available. The decision type (`Rozsudok`/`Uznesenie`/`Nalez`) may be secondary text, never the primary label; missing date → drop only the date segment.

If no citation is composable, use a descriptive fallback — never the raw document ID.

### Examples

**Correct:**
```
[40/1964 Zb. Občiansky zákonník, § 123](https://codexis.ai/sources/api/SK/ezbierka/doc/SKEZ1234/attachment/content_1.pdf#page=45)

[OS Bratislava I - 1C/123/2024 - 15.03.2024](https://codexis.ai/sources/api/SK/vseobecne-sudy/doc/SKVS5678/attachment/content_1.pdf#page=1)
```

**Incorrect:**
```
SKEZ1234 — wrong, raw document ID
cdx-sk://doc/SKEZ1234/text — wrong, shown to user as a link
https://search.example.com/api/SK/ezbierka/doc/SKEZ1234 — wrong, bare API doc URL
```

## Hard Rules

### Always Link to Attachments

Every document reference in user-facing output MUST be a clickable attachment link. Never mention a document as plain text when you have the data to build a link.

- Search results include a ready-made resolved `https://…/attachment/…` URL with `#page=N` built in — use it directly as the link target.
- **SKEZ (legislation):** The resolved source URL from search already includes `#page=N`. If `pageUrl` is absent (PDF page mapping unavailable), get the filename from `/meta` assets and link without `#page`.
- **SKVS/SKNUS (court decisions):** Search-result sources include `#page=N` (from the two-stage page lookup). When fetching `/parts` or `/meta` manually for a court decision attachment, the attachment URL has no `#page` — link without it rather than omitting the link.

### Do Not Use includeAllAssets=true on /meta

The SKEZ `/meta` endpoint defaults to `includeAllAssets=false`, which returns only the current timecut's assets. This is correct for virtually all use cases. Do NOT pass `includeAllAssets=true` unless the user explicitly asks for all historical versions.

Example: Trestný zákonník has ~80 timecut versions, each with its own set of PDF assets. Passing `includeAllAssets=true` returns all of them — dozens of PDFs across all versions — which is never useful when answering a question about current law.

## Reference Files

- **`references/search-skez.md`** — Slovak legislation search (e-Zbierka): document types, validity filters, law number lookup
- **`references/search-skvs.md`** — Slovak general court decisions: court codes, decision forms, case file numbers
- **`references/search-sknus.md`** — Slovak supreme & constitutional court decisions: legal sentences, ECLI lookup
