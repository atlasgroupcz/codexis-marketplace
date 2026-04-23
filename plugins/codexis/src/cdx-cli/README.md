# cdx-cli

`cdx-cli` is a CODEXIS CLI for source-oriented search, direct `cdx://`
resource fetches, and stored OpenAPI schema fragments.

It supports three command families:

- `search` translates convenience flags or inline JSON into authenticated
  search requests
- `get` fetches any `cdx://...` resource by translating it to the configured
  CODEXIS CDX API base URL and adding authorization
- `schema` prints concise `cdx-cli`-oriented output schemas for common `get`
  endpoints such as `/meta`, `/text`, `/toc`, `/versions`, `/related`, and
  `/related/counts`

Search examples:

```bash
cdx-cli search CR --query "náhrada škody"
cdx-cli search JD --query "náhrada škody" --court "Nejvyšší soud" --limit 5
cdx-cli search <DATA_SOURCE> --query "..." [source flags]
cdx-cli search <DATA_SOURCE> '<json-payload>'
```

Get examples:

```bash
cdx-cli get cdx://doc/CR10_2025_01_01/text
cdx-cli get --dry-run cdx://doc/JD419870/meta
cdx-cli get cdx://cz_law/89/2012/meta
cdx-cli get cdx://cz_law/89/2012/text?part=paragraf1
```

Schema examples:

```bash
cdx-cli schema meta
cdx-cli schema meta JD
cdx-cli schema text
cdx-cli schema toc
cdx-cli schema versions
cdx-cli schema related
cdx-cli schema related/counts
```

For source-specific flags and payload details, run:

```bash
cdx-cli search <DATA_SOURCE> --help
cdx-cli search [JD|CR|EU|ES|SK|LT|VS|COMMENT|ALL] --help
```

Search requests are translated into:

```text
POST {CODEXIS_API_URL}/rest/cdx-api/search/<SOURCE>
```

`get` requests translate `cdx://...` to `{CODEXIS_API_URL}/rest/cdx-api/...`.
Responses are streamed to stdout. The top-level `schema` command prints concise
endpoint schemas from the `cdx-cli` caller perspective: query parameters plus
the output shape you get back. For source-specific searches, `availableFilters`
are hidden by default unless you request facet output explicitly.

## Requirements

- `curl` must be available in `PATH`
- `CODEXIS_API_URL` must be set
- `CDX_API_JWT_AUTH` must be set

Schema commands do not require API configuration.

Example:

```bash
export CODEXIS_API_URL="https://app.codexis.cz"
export CDX_API_JWT_AUTH="Bearer <jwt>"
```

If one or both variables are missing from the process environment, `cdx-cli`
tries `~/.cdx/.env`. Process environment values win over file values.

## Build outputs

- Linux builds produce `cdx-cli`
- Linux builders with `osxcross` at `$HOME/opt/macos/target` also produce
  `cdx-cli-aarch64-apple-darwin` for modern Apple Silicon Macs
- Apple Silicon macOS builds produce `cdx-cli-aarch64-apple-darwin`
- On macOS, Docker is optional and only needed if you also want the extra Linux
  ARM64 artifact (`cdx-cli-arm64`)

`CDX_API_JWT_AUTH` accepts:

- `Authorization: Bearer <jwt>` as-is
- `Bearer <jwt>` and prefixes it with `Authorization:`
- raw JWT-like `a.b.c` values and converts them to `Authorization: Bearer ...`
- any other value as `Authorization: <value>`

## Usage

```bash
cdx-cli get cdx://doc/CR10_2025_01_01/text
cdx-cli get --dry-run cdx://doc/JD419870/meta
cdx-cli get cdx://doc/CR10_2025_01_01/toc
cdx-cli get cdx://doc/CR10_2025_01_01/versions
cdx-cli get 'cdx://doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&limit=10'
cdx-cli get cdx://doc/CR26785/related/counts
cdx-cli get cdx://cz_law/89/2012/meta
cdx-cli get cdx://cz_law/89/2012/toc
cdx-cli get cdx://cz_law/89/2012/text?part=paragraf1
cdx-cli get cdx://cz_law/89/2012/versions
cdx-cli get 'cdx://cz_law/89/2012/related?type=SOUVISEJICI_JUDIKATURA&limit=10'
cdx-cli get cdx://cz_law/89/2012/related/counts
cdx-cli search <DATA_SOURCE> --query "..." [flags]
cdx-cli search <DATA_SOURCE> '<json-payload>'
cdx-cli search <DATA_SOURCE> --query "..." '<json-payload>'
cdx-cli search <DATA_SOURCE> -
cdx-cli search <DATA_SOURCE> --dry-run --query "..."
cdx-cli search <DATA_SOURCE> --with-facets --query "..."
cdx-cli search <DATA_SOURCE> --with-full-facets --query "..."
cdx-cli schema meta
cdx-cli schema meta JD
cdx-cli schema text
cdx-cli schema toc
cdx-cli schema versions
cdx-cli schema related
cdx-cli schema related/counts
```

Common `get` resource patterns:

- `cdx://doc/<DOC_ID>/meta`
- `cdx://doc/<DOC_ID>/toc`
- `cdx://doc/<DOC_ID>/text`
- `cdx://doc/<DOC_ID>/versions` for CR documents only
- `cdx://doc/<DOC_ID>/related`
- `cdx://doc/<DOC_ID>/related/counts`
- `cdx://cz_law/<NUM>/<YEAR>/meta`
- `cdx://cz_law/<NUM>/<YEAR>/toc`
- `cdx://cz_law/<NUM>/<YEAR>/text`
- `cdx://cz_law/<NUM>/<YEAR>/versions`
- `cdx://cz_law/<NUM>/<YEAR>/related`
- `cdx://cz_law/<NUM>/<YEAR>/related/counts`

Rules:

- `get` accepts only `cdx://...` URLs and translates them to
  `{CODEXIS_API_URL}/rest/cdx-api/...`
- `cdx://` maps to `{CODEXIS_API_URL}/rest/cdx-api`
- a leading slash after `cdx://` is ignored, so `cdx:///doc/CR10/text` works
- `/versions` is supported for CR documents only
- `cdx://cz_law/.../text` supports repeated `part` query parameters
- `cdx://cz_law/.../related` supports `type`, `part`, `offset`, `limit`,
  `sort`, and `order` query parameters
- `cdx://cz_law/.../related/counts` supports optional `part`
- `<DATA_SOURCE>` is one of `CR`, `SK`, `JD`, `ES`, `EU`, `LT`, `VS`,
  `COMMENT`, `ALL`
- either `--query` or `JSON_PAYLOAD` must provide a non-empty final `"query"`
  string
- `JSON_PAYLOAD` must be a JSON object
- pass `-` instead of the payload to read JSON from stdin
- convenience flags are translated into JSON fields
- if both flags and `JSON_PAYLOAD` are provided, matching keys from JSON win
- source-native flags still work as aliases for the friendlier names
- search defaults are `limit=10`, `offset=1`, `--sort RELEVANCE`, and
  `--sort-order DESC`
- date filters use `YYYY-MM-DD`
- JSON boolean filters use `true` / `false`
- JSON sort fields should use `sort` and `sortOrder` across all sources
- CLI boolean filters are presence-only flags, for example `--current`
- backend-specific request fields are mapped internally, for example CR `sort`
  -> `sortBy`
- default search output hides top-level `availableFilters`
- `--with-facets` keeps `availableFilters` in the response for source-specific
  searches except `ALL`
- `--with-full-facets` keeps `availableFilters` and requests
  `?fullFacets=true`
- HTTP 4xx/5xx responses return a non-zero exit code while still printing the
  response body
- `schema [meta|text|toc|versions|related|related/counts]` prints concise
  caller-oriented schemas for those `get` endpoints
- `schema meta [CR|SK|JD|ES|EU|LT|VS|COMMENT]` narrows `/meta` to one
  source-specific metadata payload

Examples:

```bash
cdx-cli get cdx://doc/CR10_2025_01_01/text

cdx-cli get --dry-run cdx://doc/JD419870/meta

cdx-cli get 'cdx://doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&limit=3'

cdx-cli get cdx://cz_law/89/2012/meta

cdx-cli get cdx://cz_law/89/2012/versions

cdx-cli get cdx://cz_law/89/2012/toc

cdx-cli get cdx://cz_law/89/2012/text?part=paragraf1

cdx-cli get 'cdx://cz_law/89/2012/related?type=SOUVISEJICI_JUDIKATURA&limit=3'

cdx-cli get cdx://cz_law/89/2012/related/counts

cdx-cli search JD --query "náhrada škody" --court "Nejvyšší soud" --type Rozsudek --limit 5

cdx-cli search CR --query "občanský zákoník" --type Zákon --current --limit 5

cdx-cli search EU --query GDPR --type Nařízení --series L --limit 5

cdx-cli search JD --query "náhrada škody" '{"limit":1,"query":"náhrada škody"}'

cat payload.json | cdx-cli search EU -

cdx-cli search ALL --dry-run --query "insolvence" --limit 5

cdx-cli search JD --with-facets --query "náhrada škody" --limit 1

cdx-cli search JD --with-full-facets --query "náhrada škody" --limit 1

cdx-cli schema meta

cdx-cli schema meta JD

cdx-cli schema text

cdx-cli schema related/counts
```

## Help Model

The CLI is intentionally nested:

```bash
cdx-cli --help
cdx-cli get --help
cdx-cli schema --help
cdx-cli schema related/counts
cdx-cli search --help
cdx-cli search JD --help
```

`cdx-cli --help` and `cdx-cli get --help` show the common document resource
suffixes plus direct `cdx://cz_law/<NUM>/<YEAR>/...` examples.
`cdx-cli schema --help` lists the endpoint schema topics.
`cdx-cli schema <ENDPOINT>` prints a short plain-English explanation first, then
`---`, then the output JSON schema for that endpoint family.
`cdx-cli schema meta` points you to the source-specific `/meta` variants.
`cdx-cli schema meta <DATA_SOURCE>` prints the actual source-specific metadata
schema.
`cdx-cli search --help` lists the supported data sources.
`cdx-cli search <DATA_SOURCE> --help` shows the available flags for that source plus
a brief example request and the relevant filter formats.

## Stored Resource Schemas

Resource endpoint bundles are stored under `schemas/resource/`. They are
embedded into the binary at build time and generated from OpenAPI, then
rendered into a shorter `cdx-cli`-oriented view at runtime: brief CLI-facing
notes first, standalone JSON Schema underneath.

Refresh them from the live OpenAPI docs with:

```bash
./scripts/update-search-schemas.sh
```

The script fetches `https://beta.next.codexis.cz/rest/v3/api-docs` and falls
back to the non-`/rest` variant if needed. You can override the source with
`CODEXIS_API_DOCS_URL=<url>`.

## Supported Search Sources

- `CR` - Czech Legislation: Czech laws, decrees, regulations, and municipal documents
- `SK` - Slovak Legislation: Slovak laws and regulations
- `JD` - Czech Case Law: Judicial decisions from Czech courts
- `ES` - EU Court Decisions: EU Court of Justice and ECHR rulings
- `EU` - EU Legislation: EU regulations, directives, and decisions
- `LT` - Legal Literature: legal publications and articles
- `VS` - Contract Templates: contract specimens and templates
- `COMMENT` - Legal Commentaries: LIBERIS legal commentaries on Czech legislation
- `ALL` - Global Search: exploratory search across all sources; use only for orientation

## Notes

- `ALL` is useful for orientation, not for final authoritative retrieval.
- `cdx-cli get` remains the direct path for `/doc/...` and `/cz_law/...`
  endpoints such as `/meta`, `/text`, `/toc`, `/versions`, `/related`, and
  `/related/counts`.
