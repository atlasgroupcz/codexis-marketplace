---
uuid: 43dc8c25-4d0b-4983-a0f6-2b0bde63a852
name: cdx-at
description: This skill should be invoked whenever user needs Austrian law or legal information — federal law (Bundesrecht), case law (Judikatur), consolidated law history, state law (Landesrecht), or other publications (Sonstige) from the RIS system.
version: 2.2.0
i18n:
  cs:
    displayName: "Rakouské právo (RIS)"
    summary: "Rakouský RIS — spolková (Bundesrecht) a zemská (Landesrecht) legislativa, judikatura (Judikatur) ze 7 soudů včetně VfGH a VwGH, další úřední publikace (Sonstige) a historické konsolidované předpisy."
  en:
    displayName: "Austrian Legal Database (RIS)"
    summary: "Austria's RIS — federal (Bundesrecht) and state (Landesrecht) legislation, case law (Judikatur) from 7 courts including VfGH and VwGH, other official publications (Sonstige), and historical consolidated norms."
  sk:
    displayName: "Rakúske právo (RIS)"
    summary: "Rakúsky RIS — spolková (Bundesrecht) a krajinská (Landesrecht) legislatíva, judikatúra (Judikatur) zo 7 súdov vrátane VfGH a VwGH, ďalšie úradné publikácie (Sonstige) a historické konsolidované predpisy."
---

# Austrian Legal Database (cdx-at)

Austrian legal database (RIS — Rechtsinformationssystem) providing structured access to five sources: federal legislation published in the Bundesgesetzblatt (ATBR), court decisions from 7 courts including VfGH and VwGH (ATJD), consolidated state/provincial legislation (ATLR), directives and other official publications (ATSO), and historical consolidated federal law norms with amendment chains (ATHI).

## Commands

### search
Search documents: `cdx-at search <SOURCE> [OPTIONS]`

Use `cdx-at search <SOURCE> --help` for available filters.

### get
Fetch a document resource: `cdx-at get <cdx-at://URL> [--dry-run]`

URL families:
- `cdx-at://doc/<ID>/{meta,text,attachment/<file>}` — one document
- `cdx-at://law/<LAW_KEY>/{at?date=,versions,toc,parts,paragraph/<P>/versions,text}` — consolidated law (ATHI) with point-in-time access; `<LAW_KEY>` = Gesetzesnummer or `eli~<stem>`
- `cdx-at://{bgbl,lgbl,by-ecli,by-document-number/<domain>}/…` — resolve a gazette number / ECLI / document number to a document

Use `cdx-at get --help` for the full pattern list.

### schema
Print response schema for get endpoints: `cdx-at schema <ENDPOINT> [SOURCE]`

Use `cdx-at schema --help` for available endpoints.

## Routing: pick the right endpoint for the intent

`search` is for **discovery** — finding documents you don't yet know exist. For precise queries about a known law or paragraph, go directly to the law / document surface. Defaulting to `search` returns text-relevance-ranked §-versions that will often surface neighbouring or outdated paragraphs.

| User intent | URL pattern |
|---|---|
| Current text of § N of a known law | `cdx-at://law/<KEY>/text?part=§ N` |
| Which paragraphs are in force on date D | `cdx-at://law/<KEY>/at?date=D` |
| What § N said on date D (historical) | `cdx-at://law/<KEY>/text?part=§ N&date=D` |
| Timeline of all version dates | `cdx-at://law/<KEY>/versions` |
| All versions of one paragraph | `cdx-at://law/<KEY>/paragraph/<§>/versions` |
| Table of contents / structure | `cdx-at://law/<KEY>/toc?all=true` (omit `all=` for in-force ToC) |
| Search within ONE specific law | `cdx-at://law/<KEY>/parts?search=…&date=D` |
| Case law citing a known provision | `cdx-at://doc/<ID>/related?type=CITING_DECISION` (**not** `search` on ATJD) |
| Resolve a known NOR / BGBl / ECLI / document number | `cdx-at://by-document-number/<domain>/<dn>`, `cdx-at://bgbl/...`, `cdx-at://lgbl/...`, `cdx-at://by-ecli/<ecli>` |
| Open-ended discovery ("find laws about X") | `cdx-at search <SOURCE> --query "…"` |

`<KEY>` is either an all-digits Gesetzesnummer (e.g. `10008147` = ASVG) or `eli~<stem>` (e.g. `eli~jgs~1811~946` = ABGB) — see `references/search-athi.md` for examples.

## User-Facing Output Rules

All responses shown to the user **must** follow these formatting rules. The raw identifiers, codes, and enums from the API are for constructing CLI calls only — they must never leak into user-visible text.

### Link Format

User-facing document links MUST use the `https://` URL that appears in tool output. The binary resolves the PDF attachment of each result to a real `https://…/attachment/…` link before you see it — that resolved link is the citable **source**; link to it: `[Title](https://…)`.

For ATBR, ATJD, ATLR, and ATSO sources, search results include `#page=N` in the resolved URL when a page mapping is available — use that URL as-is: `[Title](https://…#page=N)`.

For ATHI (history) sources, search results provide document-level sources without `#page` — the history surface is a query-time computed temporal snapshot, not a paged PDF. Link to the resolved URL as-is, without adding a page suffix.

A `cdx-at://` link is NOT a user-facing link — it is an internal address you dereference with `cdx-at get cdx-at://…` to fetch more (`/meta`, `/text`, `/toc`, `/law/…`). Never show a `cdx-at://` link to the user; if you need its content, fetch it and continue.

Never resolve URLs yourself and never read `$CODEXIS_PLUGIN_AT_API_URL` for link construction. Strip `<mark>` tags from titles.

### Forbidden Raw Identifiers

Never include any of the following in user-facing text:

- Raw document IDs (e.g., `ATBR1234`, `ATJD5678`)
- Raw search prefixes (e.g., `ATBR`, `ATJD`, `ATHI`)
- Bare API URLs you construct yourself (e.g., `https://…/api/AT/judikatur/doc/ATJD123`) — cite ONLY the `https://…/attachment/…` link that appears in tool output
- Environment variable names (e.g., `$CODEXIS_PLUGIN_AT_API_URL`)
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
- **ATJD:** `court` + case number from search results
- **ATHI:** `shortTitle` or `abbreviation` + `articleParagraph` (e.g., "StGB § 165")
- **ATLR/ATSO:** `title` from search results

If title is unavailable, use `documentNumber` or a descriptive fallback — never the raw document ID.

### Examples

**Correct:**
```
[StGB § 165 — Betrügerischer Datenverarbeitungsmissbrauch](https://codexis.ai/sources/api/AT/history/doc/ATHI1234/attachment/content_1.pdf)

[VfGH, G 47/2024](https://codexis.ai/sources/api/AT/judikatur/doc/ATJD5678/attachment/content_1.pdf#page=1)

[BGBl. I Nr. 58/2018](https://codexis.ai/sources/api/AT/bundesrecht/doc/ATBR9012/attachment/content_1.pdf#page=1)
```

**Incorrect:**
```
ATJD5678 — wrong, raw document ID
cdx-at://doc/ATJD5678/text — wrong, API endpoint shown as a link
cdx-at://doc/ATJD5678/attachment/content_1.pdf — wrong, internal handle shown as a link
https://search.example.com/api/AT/judikatur/doc/ATJD5678 — wrong, bare API doc URL (not an attachment link)
```

## Hard Rules

### Always Link to Attachments

Every document reference in user-facing output MUST be a clickable attachment link. Never mention a document as plain text when you have the data to build a link.

- Search results include a ready-made resolved `https://…/attachment/…` URL — use it directly as the link target. The binary has already resolved it to a complete `https://` URL.
- For ATBR, ATJD, ATLR, ATSO: the resolved URL includes `#page=N` when a page mapping is available. Use it as-is.
- For ATHI: the resolved URL is document-level (no `#page`). Link to it as-is — do not add a page suffix.
- When a resolved URL is absent (the field is omitted from JSON when unavailable), get the filename from `/meta` assets and link without `#page` rather than omitting the link.

### Never Scrape RIS — cdx-at Is the Only Access Path

`cdx-at` is the complete, authoritative interface to RIS. **Never** fetch from or search
`ris.bka.gv.at` (or any web search engine) directly to find, open, or "verify" a document:

- Do NOT use `curl`, `wget`, `requests`, `urllib`, a headless browser, or DuckDuckGo/Google
  `site:ris.bka.gv.at` queries.
- `cdx-at search` finds documents; `cdx-at get <cdx-at://…>` fetches their content and `/meta`.
  Trust those results — the resolved `https://` link you see in tool output is already correct.
  There is no need to confirm it against the live site.
- If `cdx-at search` returns no hit, refine the query (try ATBR/ATHI/ATJD, abbreviations,
  gazette numbers) — do not fall back to scraping.

## Reference Files

- **`references/search-atbr.md`** — Austrian federal legislation search (Bundesrecht): gazette numbers, document types; BGBl resolver
- **`references/search-atjd.md`** — Austrian case law search (Judikatur): courts, decision types, ECLI, case numbers; ECLI resolver
- **`references/search-atlr.md`** — Austrian state legislation search (Landesrecht): state, gazette numbers; LGBl resolver
- **`references/search-atso.md`** — Austrian other official publications search (Sonstige): document types
- **`references/search-athi.md`** — Austrian consolidated law (History): search filters, plus the consolidated-law / point-in-time `cdx-at://law/<LAW_KEY>` surface (versions, as-of-date, paragraph history)
