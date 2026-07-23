# ATBR Search Filter Values

Austrian federal legislation (BGBl). ~18,500 documents from 2004 to present.

## query
Free-text search in title, short title, and content. Optional.

## documentType
Document type (exact match):
- `Bundesgesetz` - Federal law (Teil I)
- `Verordnung` - Regulation (Teil II)
- `Kundmachung` - Announcement (Teil III)
- `Sonstiges` - Other

## part
BGBl part (exact match):
- `Teil1` - Teil I (laws)
- `Teil2` - Teil II (regulations)
- `Teil3` - Teil III (international)

## gazetteNumber
Exact BGBl number. Example: `BGBl. II Nr. 352/2019`

## dateFrom / dateTo
Publication date range (YYYY-MM-DD, inclusive).

## sort (query param)
- `relevance` (default)
- `title`
- `date` (sorts by publication date)

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.

## Response notes
Results are collapsed by gazette publication: each matching publication appears once even when multiple fassung-date re-renders of the same act match the query. Per-hit `id` is a representative fassung's docId (use it with `/doc/{docId}/meta` to drill in). `totalResults` is the count of **distinct matching publications**, not the raw fassung-record hit count.

## Document retrieval

```
cdx-at get cdx-at://doc/ATBR1234/meta
cdx-at get cdx-at://doc/ATBR1234/text
cdx-at get cdx-at://doc/ATBR1234/attachment/content_1.pdf
```

`/meta` returns a `RisMetaResponse` envelope, not the raw metadata:
- `docId`
- `metadata` — flat scalar fields only (array-valued fields, plus `assets` and `schemaVersion`, are stripped)
- `relationCounts` — bare map keyed by relation type: `CITED_LAW`, `RELATED_BGBL`
- `assets[]` — each with `file`, `original_name`, `download_url` (a `cdx-at://doc/<docId>/attachment/<file>` link)

## Natural-key resolvers

Resolve a natural key to its ATBR docId. Each returns `{docId, domain, url}`.

```bash
cdx-at get cdx-at://by-document-number/bundesrecht/NOR40277843
cdx-at get cdx-at://bgbl/I/2026/62                 # TYPE is RIS-cased I / II / III
```
