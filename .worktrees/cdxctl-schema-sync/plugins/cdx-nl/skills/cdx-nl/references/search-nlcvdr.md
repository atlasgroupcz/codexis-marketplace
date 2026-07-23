# Search: NLCVDR (Dutch Decentralized Regulations — CVDR)

Endpoint: `cdx-nl search NLCVDR [filters]`

CVDR = Centrale Voorziening Decentrale Regelgeving: consolidated regulations of
Dutch municipalities (gemeenten), provinces, water boards (waterschappen), joint
arrangements, and Caribbean public bodies. One record per **workId**, holding all
historical versions (toestanden) as time-cuts. Results collapse to **one hit per
regulation** (latest version surfaced); `totalResults` counts distinct works.

## --query
Fulltext query. DOCUMENT mode (default) matches title^3, onderwerp and betreft.
With `--search-pages` it matches the per-page consolidated-text markdown instead.
  cdx-nl search NLCVDR --query "algemene plaatselijke verordening"

## --work-id
Exact CVDR work identifier. Applies in both DOCUMENT and PAGE mode.
  cdx-nl search NLCVDR --work-id CVDR72510

## --type
Filter by `dcterms:type` (DOCUMENT mode only). Almost always `regeling`.

## --organisatie
Issuing body, exact match against the record's organisatie (DOCUMENT mode only).
  cdx-nl search NLCVDR --organisatie Amsterdam

## --organisatie-type
Issuing-body type (DOCUMENT mode only). Exactly 7 PascalCase values:
  Gemeente — municipality
  Provincie — province
  Waterschap — water board
  Koninkrijksdeel — kingdom part
  CaribischOpenbaarLichaam — Caribbean public body (BES)
  NederlandseAntillen — pre-2010 Antilles/BES records
  RegionaalSamenwerkingsorgaan — joint regional arrangement (gemeenschappelijke regeling)
  cdx-nl search NLCVDR --organisatie-type Waterschap

## --rechtsgebied  (repeatable)
Legal-area filter (any-of match, DOCUMENT mode only). Sourced from
`dcterms:subject`; values are free-form Dutch subject strings.
  cdx-nl search NLCVDR --rechtsgebied "openbare orde en veiligheid"

## --valid-at
Only versions in force on this date (YYYY-MM-DD): validFrom ≤ date AND
(validTo > date OR validTo absent). Applies in both modes.
  cdx-nl search NLCVDR --valid-at 2024-01-01

## --valid-from-from / --valid-from-to
Range filter on the version's validFrom (YYYY-MM-DD). Applies in both modes.

## --search-pages
Switch fulltext from document metadata to per-page consolidated-text markdown.
Use when `--query` targets specific article wording. NOTE: PAGE docs do not carry
the denormalised main fields, so `--type`, `--organisatie`, `--organisatie-type`
and `--rechtsgebied` are ignored when `--search-pages` is set (work-id, valid-at
and valid-from ranges still apply).

## --offset / --limit
Pagination. Backend defaults: offset=0, limit=20 (max 100). Omit to use defaults.

## --sort / --order
sort: relevance | date | title.  order: asc | desc.
`date` sorts by validFrom; `title` sorts by the raw title. These are URL query
params on /search (backend @RequestParam), not body fields.

## Examples
  cdx-nl search NLCVDR --query "hondenbelasting" --organisatie-type Gemeente
  cdx-nl search NLCVDR --work-id CVDR72510
  cdx-nl search NLCVDR --query "ligplaats" --valid-at 2024-01-01 --limit 20
  cdx-nl search NLCVDR --query "geluidhinder" --search-pages --sort date --order desc

# Worked example: resolving a CVDR work-id directly

When you already know the CVDR work-id (e.g. from an external citation), skip
`search` — the workid route resolves it to a display ID and the latest version:

```
cdx-nl get "cdx-nl://workid/CVDR72510"
#   → { workId, docId, domain, recordId, title, validFrom, url, docUrl }
#   docUrl is a ready-to-fetch cdx-nl://doc/NLCVDR.../meta link.
```

# Working with CVDR documents

CVDR documents are versioned (N toestanden per work). Search returns the latest
version; use `/versions` or `/at` to navigate time, and pass `?version=<label>`
to `/text`, `/toc`, `/parts` to pin a specific one. The `version` label is the
manifest expression label carried on each version item (NOT a date).

```
DOC_ID="NLCVDR1234"

# Metadata (work-level) + attachments (assets[].download_url are resolved https links):
cdx-nl get "cdx-nl://doc/${DOC_ID}/meta"

# All versions, latest → oldest by validFrom:
cdx-nl get "cdx-nl://doc/${DOC_ID}/versions" | jq '.items[] | {version, validFrom, validTo}'

# The single version in force on a date (+ its PDF, bijlagen, neighbour labels, withdrawn flag):
cdx-nl get "cdx-nl://doc/${DOC_ID}/at?date=2024-01-01"

# Full consolidated text (defaults to the currently valid version):
cdx-nl get "cdx-nl://doc/${DOC_ID}/text"
cdx-nl get "cdx-nl://doc/${DOC_ID}/text?version=2"
```

### Finding one article without downloading the whole regulation

Two calls: `/parts?search=` to locate the article id, then `/text?part=` to read
just that article. `?part=` is repeatable: `text?part=a&part=b`.

```
# 1. Find the part id (filters by id/nr/title substring):
cdx-nl get "cdx-nl://doc/${DOC_ID}/parts?search=3"
#   → items[] with { id, nr, title, kind, startPage, startSourceFile }

# 2. Read just that article's text:
cdx-nl get "cdx-nl://doc/${DOC_ID}/text?part=<id-from-step-1>"
```

`/toc` returns the same flat article list as a TOC.

### Related laws (grondslag)

CVDR relations are **outgoing-only** legal-basis (grondslag) references — the
laws a regulation is based on. `direction=in` returns empty.

```
cdx-nl get "cdx-nl://doc/${DOC_ID}/related?direction=out" | jq '.items[] | {raw, bwbId, kind}'
```

Each item is `{ raw, bwbId, href, kind }`. `kind=bwb` items carry a resolved
`bwbId` and an `href` to the BWB `/bwbid/{bwbId}` resolver (chase it via the
NLBWB surface); `kind=external` items are unresolved deeplinks — use `raw`/`href`
as-is, do not construct cdx-nl URLs yourself.

### Attachments

Attachment filenames come from `meta.assets[]` (and each version's `pdf` /
`bijlagen` refs). Download via the resolved `download_url`, or:

```
cdx-nl get "cdx-nl://doc/${DOC_ID}/attachment/content_1.pdf"
```
