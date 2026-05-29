# Search: NLBWB (Dutch National Legislation)

Endpoint: `cdx-nl search NLBWB [filters]`

## --query
Fulltext query across title and (if `--search-pages`) page-level markdown.
  cdx-nl search NLBWB --query "Burgerlijk Wetboek"

## --bwb-id
Exact BWB document identifier.
  cdx-nl search NLBWB --bwb-id BWBR0001827

## --afkorting
Filter by abbreviation, matched **exactly** against the record's
`metadata.main.afkorting`. Only useful when the abbreviation IS the record's
exact afkorting (e.g. `WVW 1994` for the Wegenverkeerswet). For multi-Boek
codes (`BW`, `Sr`, `Rv`, `Awb`) no record carries that bare abbreviation —
this filter will return zero hits. Use the `/afkorting/<abbr>/meta` resolver
instead (see "Resolving by afkorting" below).
  cdx-nl search NLBWB --afkorting "WVW 1994"

## --type
BWB type letter (NOT regelingSoort). Allowed values:
  R — Regelgeving
  V — Verdragen
  W — Wetgeving

## --regeling-soort
Regeling soort. Backend example values: "Wet", "Besluit", "Verdrag".

## --issuing-authority
Issuing authority, exact match (e.g. "Ministerie van Justitie en Veiligheid").

## --rechtsgebied  (repeatable)
Legal area filter (any-of match). Examples:
  cdx-nl search NLBWB --rechtsgebied "burgerlijk recht" --rechtsgebied "huurrecht"

## --valid-at
Only laws valid on this date (YYYY-MM-DD).
  cdx-nl search NLBWB --valid-at 2026-01-01

## --valid-from-from / --valid-from-to
Range filter on the law's first valid-from date.

## --search-pages
Switch fulltext to per-page markdown. Use when --query targets specific articles.
Required for --pdf-kind to take effect.

## --pdf-kind
`consolidated` | `stb_publication`. Restrict matches to a specific PDF kind.
Only effective when --search-pages is also set.

## --offset / --limit
Pagination. Backend defaults: offset=0, limit=20 (max 100). Omit flags to use those defaults.

## --sort / --order
sort: relevance | date.  order: asc | desc.
URL query params on /search (backend @RequestParam), not body fields.

## Examples
  cdx-nl search NLBWB --query "burgerlijk wetboek"
  cdx-nl search NLBWB --bwb-id BWBR0001827 --valid-at 2026-01-01
  cdx-nl search NLBWB --rechtsgebied huurrecht --limit 20 --sort date --order desc

# Resolving by afkorting

For common Dutch law abbreviations, prefer the `/afkorting/<abbr>/` GET routes
over `search --afkorting`. Search filters require an EXACT match against the
record's `metadata.main.afkorting`; the GET routes ALSO consult a curated alias
table so multi-Boek codes (`BW`, `Sr`, `Rv`, `Awb`, `Wvw`) resolve to a
canonical bwbId even though no record uses those bare strings.

Keys are matched case-insensitively. `BW` resolves to Burgerlijk Wetboek
Boek 1 by default — for a specific Boek, use the bwb-id route.

  cdx-nl get cdx-nl://afkorting/BW/meta
  cdx-nl get cdx-nl://afkorting/Sr/versions?limit=5
  cdx-nl get cdx-nl://afkorting/Awb/at?date=2024-01-01

# Worked example: find one article without downloading the whole law

Two calls are usually enough — `/parts?search=` to locate the article id, then
`/text?part=` to read its body. Skip `/text` (without `?part=`) when you only
need one or two articles; a full BWB toestand is megabytes.

```
# 1. Find article 1:1 BW (legal personality):
cdx-nl get "cdx-nl://afkorting/BW/parts?search=1:1"
#   → returns parts[] with { id, nr, title, startPage, startSourceFile }
#   The `id` is the article's XML id and is opaque — just pass it back as-is.

# 2. Read just that article's text:
cdx-nl get "cdx-nl://afkorting/BW/text?part=<id-from-step-1>"

# 3. Build the user-facing PDF link from the same parts[] entry:
#   cdx-nl://doc/<docId>/attachment/<startSourceFile>#page=<startPage>
```

`?part=` is repeatable — pass multiple ids to concatenate their bodies in one
call: `text?part=A&part=B&part=C`.

`/parts?search=` and `/text?part=` are available on all three prefixes
(`/doc/<id>/`, `/bwbid/<bwbId>/`, `/afkorting/<abbr>/`) and accept the same
optional `toestandId` query param (defaults to the latest toestand).

# Get endpoints (NLBWB only)

Beyond /meta, /text, /toc, /parts, /related, /citations (shared with NLUIT where
applicable), NLBWB exposes three time-axis / reverse-citation endpoints. Run
`cdx-nl schema <endpoint>` for the full response schema; the notes below are a
quick map.

## /versions
List toestanden, latest → oldest by validFrom. Per-item shape carries the
consolidated `pdf` separately from `amendmentPdfs[]` (Stb/Stcrt/Trb).

Query params: `limit` (default 10, max 100), `offset`, `from` / `to`
(YYYY-MM-DD validFrom window), `includeAll=true` (ETL — bypass paging).

Response envelope: `{ docId, total, returned, versions: [...] }`.

  cdx-nl get "cdx-nl://doc/NLBWB1234/versions?limit=5"
  cdx-nl get "cdx-nl://afkorting/BW/versions?from=2020-01-01&to=2024-12-31"

## /at
Resolve a law to its toestand in force on a given date. Use this instead of
`/versions` when you only need a single time-resolved version.

Query params: `date` (YYYY-MM-DD, **required**).

Response: `{ docId, date, toestand, pdf, amendmentPdfs[], previousToestandId,
nextToestandId, versions: { total, url } }`. 400 on missing/malformed date,
404 if the date is outside the document's coverage.

  cdx-nl get "cdx-nl://doc/NLBWB1234/at?date=2020-06-15"
  cdx-nl get "cdx-nl://law/NL/BWBR0001827/at?date=2020-06-15"

## /cited-by-decisions
Reverse lookup: Rechtspraak (NLUIT) decisions whose `lawReferences` cite this
BWB document. Sort: `decisionDate` desc, `ecli` asc (tiebreaker).

Query params: `limit` (default 10, max 100), `offset`.

Response: `{ docId, total, returned, decisions: [{ ecli, decisionDate, court,
metaUrl }] }`. Each `metaUrl` is a ready-to-fetch `cdx-nl://doc/NLUIT.../meta`.

  cdx-nl get "cdx-nl://doc/NLBWB1234/cited-by-decisions"
  cdx-nl get "cdx-nl://afkorting/BW/cited-by-decisions?limit=50"
