---
uuid: 858f1a59-06c5-46a4-84dc-62d50cc1b6af
name: cdx-nl
description: Dutch national legislation (BWB) and Dutch case law (Rechtspraak.nl) — use when the user asks about Dutch laws, Dutch court decisions, ECLI identifiers starting with ECLI:NL:..., or references BWB-ids.
version: 1.0.0
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

**IMPORTANT:** All document links in user-facing output MUST use the `cdx-nl://` scheme. The system automatically resolves these to real URLs at render time. Never resolve URLs yourself — never read or use `$CODEXIS_PLUGIN_NL_API_URL` for link construction.

Every document reference MUST be a clickable attachment link — never plain text. Attachment-link construction depends on the source (NLBWB vs NLUIT); see the **Attachment link construction** section below for the full rules.

Never present search, meta, text, toc, parts, versions, related, or citations API endpoints as clickable links — those are internal tool calls only.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `NLBWB1234`, `NLUIT5678`)
- Raw search prefixes (e.g., `NLBWB`, `NLUIT`)
- Raw BWB-ids (e.g., `BWBR0001840`) and raw ECLIs (e.g., `ECLI:NL:HR:2024:123`) outside of a wrapping link
- Resolved HTTP URLs (e.g., `https://search.example.com/api/NL/wettenbwb/doc/...`)
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
- **NLUIT:** `title` from search results, include court name and case identifier inside the link text (e.g., `[Uitspraak — Hoge Raad, ECLI:NL:HR:2024:123](cdx-nl://doc/NLUIT5678/attachment/decision.pdf)`).

If title is unavailable, use a descriptive fallback — never the raw document ID.

### Examples

**Correct:**
```
[art. 1:1 Burgerlijk Wetboek Boek 1](cdx-nl://doc/NLBWB1234/attachment/content_1.pdf#page=3)

[Uitspraak — Hoge Raad, ECLI:NL:HR:2024:123](cdx-nl://doc/NLUIT5678/attachment/decision.pdf#page=12)
```

**Incorrect:**
```
NLBWB1234 — wrong, raw document ID
cdx-nl://doc/NLBWB1234/text — wrong, API endpoint as link
https://search.example.com/api/NL/wettenbwb/doc/NLBWB1234 — wrong, resolved URL
```

## Attachment link construction

### Two sources of attachment links

Every NLBWB or NLUIT response carries asset references in two places:

1. **`/meta` → `assets[]`** — each asset has `original_name` and
   `download_url`. The `download_url` is **already a complete `cdx-nl://...`
   link** (e.g. `cdx-nl://doc/NLBWB1/attachment/content_1.pdf`). Use it
   verbatim — do NOT rebuild it from `original_name`.

2. **`/parts` → items** — for NLBWB, each part has `start_source_file`
   (consolidated PDF filename) and `start_page` (1-based page). For NLUIT,
   each part has `page` + first heading.

### NLBWB: linking to a specific page

The `download_url` from `/meta` lands on page 1. To deep-link a specific
article, take `start_source_file` + `start_page` from the matching
`/parts` entry and append `#page=`:

    cdx-nl://doc/<NLBWB_ID>/attachment/<start_source_file>#page=<start_page>

If `start_page` is null, omit `#page=`. If you haven't fetched `/parts` yet
and only need the document-level link, use `assets[].download_url` from `/meta`.

### NLUIT: linking to a specific page

Get the base link from `/meta` `assets[].download_url`. To cite a specific
page, look up the page number in `/parts` and append `#page=<N>`:

    cdx-nl://doc/<NLUIT_ID>/attachment/decision.pdf#page=12

The filename always comes from `/meta`, not `/parts`.

### IMPORTANT: only /doc/<ID>/attachment supports anonymous signed access

The backend exposes attachment paths only on `/doc/<DOC_ID>/attachment/` and
`/by-ecli/<ECLI>/attachment/` (NLUIT). There are NO attachment routes on
`/bwbid/...` or `/afkorting/...`. Always construct user-facing attachment
links with a `/doc/<DOC_ID>/` prefix — resolve via `/meta` if you only have a
BWB-id or afkorting in hand.

### Staatsblad / Staatscourant / Tractatenblad PDFs

When you need to cite a Staatsblad publication (e.g. `stb-2024-123`):

1. Call `cdx-nl get cdx-nl://publication/<id>/resolve`
2. Parse the `{docId, file}` response.
3. Emit the user-facing link `cdx-nl://doc/<docId>/attachment/<file>`.

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
  is a ready-to-fetch `cdx-nl://doc/NLUIT.../meta`.

Both endpoints accept `/doc/<NLBWB_ID>/`, `/law/NL/<BWB_ID>/`, and
`/afkorting/<ABBR>/` prefixes.

## Reference Files

- **`references/search-nlbwb.md`** — Dutch national legislation search (Basiswettenbestand): document types, validity filters, BWB-id and afkorting lookup
- **`references/search-nluit.md`** — Dutch case law search (Rechtspraak.nl): court codes, decision types, ECLI lookup
