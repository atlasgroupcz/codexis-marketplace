# ATHI Search Filter Values

Austrian consolidated federal law norms (Bundesnormen / History). ~437,000 norm provisions with amendment chains.

## query
Free-text search in title, short title, and content. Optional.

## documentType
Document type code (exact match):
- `BG` - Bundesgesetz (Federal law)
- `V` - Verordnung (Regulation)
- And other type codes used in RIS

## abbreviation
Law abbreviation (exact match). Common examples:
- `ASVG` - Allgemeines Sozialversicherungsgesetz
- `StGB` - Strafgesetzbuch
- `B-VG` - Bundes-Verfassungsgesetz
- `ABGB` - Allgemeines buergerliches Gesetzbuch
- `StPO` - Strafprozessordnung
- `ZPO` - Zivilprozessordnung
- `AVG` - Allgemeines Verwaltungsverfahrensgesetz
- `BAO` - Bundesabgabenordnung
- `EStG` - Einkommensteuergesetz
- `GewO` - Gewerbeordnung
- `KSchG` - Konsumentenschutzgesetz
- `UStG` - Umsatzsteuergesetz

## dateFrom / dateTo
Effective date range (YYYY-MM-DD, inclusive).

## sort (query param)
- `relevance` (default)
- `title`
- `date` (sorts by effective date)

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.

## Response notes
Results are collapsed by law: each matching law appears once even when many of its §-versions match the query. Per-hit `id` is a representative §-version's docId (use it with `/doc/{docId}/meta` to drill in). `totalResults` is the count of **distinct matching laws**, not the raw §-version hit count — typically much smaller than the underlying record count (a single large law like ASVG has ~10k §-versions but contributes 1 to `totalResults`).

## Document retrieval

```
cdx-at get cdx-at://doc/ATHI1234/meta
cdx-at get cdx-at://doc/ATHI1234/text
cdx-at get cdx-at://doc/ATHI1234/attachment/content_1.pdf
```

`/meta` returns a `RisMetaResponse` envelope, not the raw metadata:
- `docId`
- `metadata` — flat scalar fields only (array-valued fields, plus `assets` and `schemaVersion`, are stripped)
- `relationCounts` — bare map keyed by relation type: `CITING_DECISION`, `RELATED_BGBL`, `CITED_LAW`
- `assets[]` — each with `file`, `original_name`, `download_url` (a `cdx-at://doc/<docId>/attachment/<file>` link)

## Natural-key resolver

Resolve a NOR document number to its ATHI docId. Returns `{docId, domain, url}`.

```bash
cdx-at get cdx-at://by-document-number/history/NOR40277843
```

## Consolidated law / point-in-time (`cdx-at://law/<LAW_KEY>`)

ATHI exposes a law-keyed temporal surface: a whole consolidated law, navigable by
paragraph and as-of any date. `LAW_KEY` is either an all-digits Gesetzesnummer
(e.g. `10008147` = ASVG) or `eli~<stem>` with `/` encoded as `~` (e.g.
`eli~jgs~1811~946` = ABGB) for pre-modern law that has no Gesetzesnummer.

```
cdx-at get cdx-at://law/<LAW_KEY>                              # summary (title, currentParagraphCount, first/lastEffectiveDate)
cdx-at get cdx-at://law/<LAW_KEY>/versions                     # consolidation timeline (one entry per effectiveDate)
cdx-at get 'cdx-at://law/<LAW_KEY>/at?date=YYYY-MM-DD'         # point-in-time snapshot: paragraphs in force on that date
cdx-at get 'cdx-at://law/<LAW_KEY>/toc?all=true'              # table of contents (omit all= for in-force on date)
cdx-at get 'cdx-at://law/<LAW_KEY>/parts?date=&search=&offset=&limit='
cdx-at get 'cdx-at://law/<LAW_KEY>/paragraph/<PARA>/versions'  # every temporal version of one paragraph
cdx-at get 'cdx-at://law/<LAW_KEY>/text?date=&part=<PARA>'     # plain text; part repeatable + CSV, max 200 paragraphs without part
```

Examples:
```bash
# ASVG as it stood on 1 June 2024, just the § list
cdx-at get 'cdx-at://law/10008147/at?date=2024-06-01' | jq '.paragraphs[] | {articleParagraph, docId, effectiveDate}'

# ABGB (pre-1900 law, no Gesetzesnummer) — full table of contents
cdx-at get 'cdx-at://law/eli~jgs~1811~946/toc?all=true' | jq '.items | length'

# history of one paragraph across all amendments
cdx-at get 'cdx-at://law/10008147/paragraph/§ 5/versions' | jq '.versions[] | {effectiveDate, docId}'
```

In-force rule: for each paragraph the winning version on date `D` is the one
with the greatest `effectiveDate <= D` (tie-break: `documentNumber` desc).
`expiryDate` is ignored; paragraphs not yet effective on `D` are omitted.
Omit `?date` (or pass `all=true` on `/toc` and `/parts`) for every paragraph ever.
Each item's `docId` is a normal ATHI document — fetch its text with
`cdx-at get cdx-at://doc/<docId>/text`.
