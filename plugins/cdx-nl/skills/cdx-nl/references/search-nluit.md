# Search: NLUIT (Dutch Case Law — Rechtspraak.nl)

Endpoint: `cdx-nl search NLUIT [filters]`

## --query
Fulltext query across decision text.
  cdx-nl search NLUIT --query "huurrecht woonruimte"

## --court  (repeatable)
Court code. Common values:
  HR        — Hoge Raad
  RBAMS     — Rechtbank Amsterdam
  GHAMS     — Gerechtshof Amsterdam
  GHARL     — Gerechtshof Arnhem-Leeuwarden
  CRVB      — Centrale Raad van Beroep
  RVS       — Raad van State

  cdx-nl search NLUIT --court HR --court GHAMS

## --decision-type
Uitspraak | Conclusie | Beslissing | Tussenuitspraak | …

## --decision-date-from / --decision-date-to
Decision date range (YYYY-MM-DD).
  cdx-nl search NLUIT --decision-date-from 2024-01-01 --decision-date-to 2024-12-31

## --legal-area  (repeatable)
Legal area filter — backend field is `legalAreas` (plural). Common values:
  "Civiel recht", "Strafrecht", "Bestuursrecht", "Belastingrecht".

## --procedure
Procedure type. Common values: Cassatie, Hoger beroep, Eerste aanleg.

## --case-number
Exact case number (e.g. "21/00123"). Backend treats this as the registered
case number, NOT as ECLI. To look up by ECLI, use the dedicated resolver:
  cdx-nl get cdx-nl://ecli/ECLI:NL:HR:2024:1234

## --from / --size
Pagination (NLUIT uses from/size, not offset/limit). Backend defaults: from=0,
size=20 (max 100). Omit flags to use those defaults.

## --sort / --order
sort: relevance | date.  order: asc | desc.
These are URL query params on /search (backend @RequestParam), not body fields.

## Examples
  cdx-nl search NLUIT --query "huurrecht" --court HR --size 5
  cdx-nl search NLUIT --decision-date-from 2024-01-01 --procedure Cassatie
  cdx-nl search NLUIT --case-number "21/00123"

# Worked example: resolving an ECLI to a citable PDF in one step

When the user pastes an ECLI (or you derive one from another endpoint, e.g.
NLBWB `/cited-by-decisions`), skip `search` entirely — the ECLI route resolves
directly to the document:

```
cdx-nl get cdx-nl://ecli/ECLI:NL:HR:2024:1234/meta
#   → { docId, metadata, assets: [{ original_name, download_url, ... }] }

# Build the user-facing link from assets[].download_url (already a cdx-nl://
# attachment link). To deep-link a page, fetch /parts and append #page=:
cdx-nl get "cdx-nl://ecli/ECLI:NL:HR:2024:1234/parts"
```

The ECLI route also accepts `/text`, `/toc`, `/parts`, `/related`, and `/attachment/<filename>` suffixes — same shape as the `/doc/<id>/` route.

### Following citations (NLUIT)

A decision's citation graph is **not** in `/meta`. `/meta` carries only a small
`relationCounts` triage object:

- `relationCounts.citedLaws` — distinct BWB laws this decision cites
- `relationCounts.citedDecisions` — decisions it cites (incl. foreign CJEU/ECHR)
- `relationCounts.citingDecisions` — decisions that cite it

Read those first. Then fetch only the direction(s) worth chasing:

`cdx-nl://doc/<NLUIT_ID>/related?type=citedLaws|citedDecisions|citingDecisions&offset=&limit=`

Each page is `{ type, totalResults, offset, limit, results[] }`.

- **Law items** carry `bwbId`, `title`, `articles` (article-level pinpoints), and a `url`.
- **Decision items** carry `ecli`, `types` (the LiDO citation role(s), e.g.
  `rvr-cassatie-eerdereaanleg` = the ruling under appeal — usually one, sometimes several),
  `court`, `decisionDate`, and a `url`.
- A **`url` is present only when we can resolve the reference to one of our documents.**
  Foreign citations (CJEU `ECLI:EU:...`, ECHR `ECLI:CE:...`) appear with their `ecli` and
  `types` but **no `url`** — report them as authorities the decision relies on; do not try to
  fetch them via cdx-nl (the ECLI may be resolvable by another plugin or externally).
- Use `url` **verbatim**; never construct backend or cdx-nl URLs yourself.
- `/related/counts` returns the same counts as `/meta.relationCounts` for a lightweight,
  metadata-free poll.
