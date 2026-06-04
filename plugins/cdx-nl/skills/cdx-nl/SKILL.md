---
uuid: 858f1a59-06c5-46a4-84dc-62d50cc1b6af
name: cdx-nl
description: Dutch national legislation (BWB) and Dutch case law (Rechtspraak.nl) — use when the user asks about Dutch laws, Dutch court decisions, ECLI identifiers starting with ECLI:NL:..., or references BWB-ids.
version: 2.0.0
i18n:
  cs:
    displayName: "Nizozemské právo (BWB a Rechtspraak.nl)"
    summary: "Nizozemská právní databáze — národní legislativa z Basiswettenbestand (BWB) a rozhodnutí nizozemských soudů z Rechtspraak.nl."
  en:
    displayName: "Dutch Legal Database (BWB & Rechtspraak.nl)"
    summary: "Dutch legal database — national legislation from Basiswettenbestand (BWB) and Dutch court decisions from Rechtspraak.nl."
  nl:
    displayName: "Nederlandse rechtsdatabase (BWB & Rechtspraak.nl)"
    summary: "Nederlandse rechtsdatabase — nationale wetgeving uit Basiswettenbestand (BWB) en uitspraken van Nederlandse rechtbanken via Rechtspraak.nl."
---

# Dutch Legal Database (cdx-nl)

Dutch legal database providing structured access to two sources: national legislation from Basiswettenbestand (NLBWB) and Dutch case law from Rechtspraak.nl (NLUIT). Invoke when the user asks about Dutch laws, Dutch court decisions, ECLI identifiers starting with `ECLI:NL:...`, or references BWB-ids.

## Commands

### search
Search documents: `cdx-nl search <SOURCE> [OPTIONS]`

Use `cdx-nl search <SOURCE> --help` for available filters.

### get
Fetch a document resource: `cdx-nl get <cdx-nl://URL> [--dry-run]`

Use `cdx-nl get --help` for available URL patterns.

### schema
Print response schema for get endpoints: `cdx-nl schema <ENDPOINT> [SOURCE]`

Use `cdx-nl schema --help` for available endpoints.

## User-Facing Output Rules

All responses shown to the user **must** follow these formatting rules. The raw identifiers, codes, and enums from the API are for constructing CLI calls only — they must never leak into user-visible text.

### Link Format

User-facing document links MUST use the `https://` URL that appears in tool output. The binary resolves the PDF attachment of each result to a real `https://…/attachment/…#page=N` link before you see it — that resolved link is the citable **source**; link to it: `[Title](https://…#page=N)`.

A `cdx-nl://` link is NOT a user-facing link — it is an internal address you dereference with `cdx-nl get cdx-nl://…` to fetch more (`/meta`, `/text`, `/toc`). Never show a `cdx-nl://` link to the user; if you need its content, fetch it and continue.

Never resolve URLs yourself and never read `$CODEXIS_PLUGIN_NL_API_URL` for link construction. Strip `<mark>` tags from titles.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `NLBWB1234`, `NLUIT5678`)
- Raw search prefixes (e.g., `NLBWB`, `NLUIT`)
- Raw BWB-ids (e.g., `BWBR0001840`) and raw ECLIs (e.g., `ECLI:NL:HR:2024:123`) outside of a wrapping link
- Bare API URLs you construct yourself (e.g., `https://…/api/NL/wetten/bwb/doc/NLBWB1234`) — cite ONLY the `https://…/attachment/…` PDF link that appears in tool output
- Environment variable names (e.g., `$CODEXIS_PLUGIN_NL_API_URL`)
- HTML tags (e.g., `<a href=...>`) — use markdown links only

### Human-Readable Source Names

When referring to data sources in prose, match the user's conversation language:

| Code (internal) | Dutch Name                  | English Name         |
|-----------------|-----------------------------|----------------------|
| `NLBWB`         | Basiswettenbestand          | National Legislation |
| `NLUIT`         | Uitspraken (Rechtspraak.nl) | Case Law             |

### Document Titles

Use these fields as the link text:

- **NLBWB:** `title` from search results (e.g., "Burgerlijk Wetboek Boek 1") — strip `<mark>` tags. When citing a specific article, prefix with the article reference (e.g., `art. 1:1 BW`).
- **NLUIT:** `title` from search results, include court name and case identifier inside the link text (e.g., `[Uitspraak — Hoge Raad, ECLI:NL:HR:2024:123](https://…/attachment/decision.pdf#page=12)`).

If title is unavailable, use a descriptive fallback — never the raw document ID.

### Examples

**Correct:**
```
[art. 1:1 Burgerlijk Wetboek Boek 1](https://…/NL/wetten/bwb/doc/NLBWB1234/attachment/content_1.pdf#page=3)

[Uitspraak — Hoge Raad, ECLI:NL:HR:2024:123](https://…/NL/rechtspraak/uitspraken/doc/NLUIT5678/attachment/decision.pdf#page=12)
```

**Incorrect:**
```
NLBWB1234 — wrong, raw document ID
cdx-nl://doc/NLBWB1234/text — wrong, API endpoint as link
cdx-nl://doc/NLBWB1234/attachment/content_1.pdf#page=3 — wrong, cdx:// is an internal handle; cite the https:// link from tool output
https://search.example.com/api/NL/wetten/bwb/doc/NLBWB1234 — wrong, bare API doc URL (cite the /attachment/ PDF link from tool output instead)
```

## Hard Rules

### Always Link to Attachments

Every document reference in user-facing output MUST be a clickable attachment link. Never mention a document as plain text when you have the data to build a link.

The binary resolves `cdx-nl://…/attachment/…` links in tool output to absolute `https://` URLs before you see them. Use those resolved `https://` links directly as link targets — do not reconstruct them yourself.

When a direct `https://…/attachment/…` URL is not yet available (e.g. you only have a BWB-id or ECLI), fetch `/meta` first to get the resolved `assets[].download_url`, which will appear as an `https://` URL in the output. For a page-specific deep-link, also fetch `/parts` and append `#page=<N>`.

## Attachment Link Workflows

### NLBWB: linking to a specific page

The `download_url` from `/meta` (which appears in tool output as an `https://` URL) lands on page 1. To deep-link a specific article, take `start_source_file` + `start_page` from the matching `/parts` entry and append `#page=`:

Use the `https://` URL that appears in tool output after fetching:
- `cdx-nl get cdx-nl://doc/<NLBWB_ID>/meta` → yields resolved `https://` attachment URL
- `cdx-nl get cdx-nl://doc/<NLBWB_ID>/parts` → yields per-article page numbers

If `start_page` is null, omit `#page=`. If you haven't fetched `/parts` yet and only need the document-level link, use the `https://` URL from `assets[].download_url` in `/meta` output.

### NLUIT: linking to a specific page

Fetch `/meta` to get the resolved `https://` attachment URL. To cite a specific page, fetch `/parts` and append `#page=<N>` to the filename URL from the output.

The filename always comes from `/meta` assets output, not `/parts`.

### IMPORTANT: only /doc/<ID>/attachment supports anonymous signed access

The backend exposes attachment paths only on `/doc/<DOC_ID>/attachment/` and
`/by-ecli/<ECLI>/attachment/` (NLUIT). There are NO attachment routes on
`/bwbid/...` or `/afkorting/...`. Always construct user-facing attachment
links from the resolved `https://` URL in tool output — resolve via
`cdx-nl get cdx-nl://doc/<ID>/meta` if you only have a BWB-id or afkorting in hand.

### Staatsblad / Staatscourant / Tractatenblad PDFs

When you need to cite a Staatsblad publication (e.g. `stb-2024-123`):

1. Call `cdx-nl get cdx-nl://publication/<id>/resolve`
2. Parse the `{docId, file}` response.
3. Fetch `cdx-nl get cdx-nl://doc/<docId>/meta` to get the resolved `https://` attachment URL.
4. Emit the user-facing link using that `https://` URL.

Never try to `cdx-nl get cdx-nl://publication/<id>` — that URI is intentionally
not exposed; the resolver always returns a `/doc/.../attachment/<file>` target.

### NLBWB /citations requires toestandId

Always fetch `/versions` first if the user hasn't specified one, then default to
the latest toestand's `toestandId`.

### NLBWB time-axis and reverse-citation endpoints

Prefer these over `/versions` when applicable:

- `/at?date=YYYY-MM-DD` — single toestand in force on a given date, plus its
  PDFs and previous/next pointers. Use when the user references "as of date X"
  rather than wanting the whole history.
- `/cited-by-decisions` — Rechtspraak (NLUIT) decisions whose `lawReferences`
  cite this BWB document. Paged via `limit`/`offset`. Each `decisions[].metaUrl`
  is a ready-to-fetch `cdx-nl://doc/NLUIT.../meta` (dereference it with `cdx-nl get`).

Both endpoints accept `/doc/<NLBWB_ID>/`, `/law/NL/<BWB_ID>/`, and
`/afkorting/<ABBR>/` prefixes.

## Reference Files

- **`references/search-nlbwb.md`** — Dutch national legislation search (Basiswettenbestand): document types, validity filters, BWB-id and afkorting lookup
- **`references/search-nluit.md`** — Dutch case law search (Rechtspraak.nl): court codes, decision types, ECLI lookup
