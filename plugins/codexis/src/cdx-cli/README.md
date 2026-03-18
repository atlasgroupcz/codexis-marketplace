# cdx-cli

`cdx-cli` is a source-oriented CODEXIS search CLI.

It focuses on one job for now: translate either convenience flags or inline JSON
into the correct search request:

```bash
cdx-cli search <SOURCE> --query "..." [source flags]
cdx-cli search <SOURCE> '<json-payload>'
```

into the correct authenticated `curl` call to:

```text
POST {CODEXIS_API_URL}/rest/cdx-api/search/<SOURCE>
```

The API response is streamed to stdout as JSON.

## Requirements

- `curl` must be available in `PATH`
- `CODEXIS_API_URL` must be set
- `CDX_API_JWT_AUTH` must be set

Example:

```bash
export CODEXIS_API_URL="https://app.codexis.cz"
export CDX_API_JWT_AUTH="Bearer <jwt>"
```

If one or both variables are missing from the process environment, `cdx-cli`
tries `~/.cdx/.env`. Process environment values win over file values.

`CDX_API_JWT_AUTH` accepts:

- `Authorization: Bearer <jwt>` as-is
- `Bearer <jwt>` and prefixes it with `Authorization:`
- raw JWT-like `a.b.c` values and converts them to `Authorization: Bearer ...`
- any other value as `Authorization: <value>`

## Usage

```bash
cdx-cli search <SOURCE> --query "..." [flags]
cdx-cli search <SOURCE> '<json-payload>'
cdx-cli search <SOURCE> --query "..." '<json-payload>'
cdx-cli search <SOURCE> -
cdx-cli search <SOURCE> --dry-run --query "..."
```

Rules:

- `<SOURCE>` is one of `ALL`, `COMMENT`, `CR`, `ES`, `EU`, `JD`, `LT`, `SK`, `VS`
- either `--query` or `JSON_PAYLOAD` must provide a non-empty final `"query"` string
- `JSON_PAYLOAD` must be a JSON object
- pass `-` instead of the payload to read JSON from stdin
- convenience flags are translated into JSON fields
- if both flags and `JSON_PAYLOAD` are provided, matching keys from JSON win
- source-native flags still work as aliases for the friendlier names
- search defaults are `limit=10`, `offset=1`, `--sort RELEVANCE`, and `--sort-order DESC`
- date filters use `YYYY-MM-DD`
- JSON boolean filters use `true` / `false`
- JSON sort fields should use `sort` and `sortOrder` across all sources
- CLI boolean filters are presence-only flags, for example `--current`
- backend-specific request fields are mapped internally, for example CR `sort` -> `sortBy`

Examples:

```bash
cdx-cli search JD --query "náhrada škody" --court "Nejvyšší soud" --type Rozsudek --limit 5

cdx-cli search CR --query "občanský zákoník" --type Zákon --current --limit 5

cdx-cli search EU --query GDPR --type Nařízení --series L --limit 5

cdx-cli search JD --query "náhrada škody" '{"limit":1,"query":"náhrada škody"}'

cat payload.json | cdx-cli search EU -

cdx-cli search ALL --dry-run --query "insolvence" --limit 5
```

## Help Model

The CLI is intentionally nested:

```bash
cdx-cli --help
cdx-cli search --help
cdx-cli search JD --help
```

`cdx-cli search --help` lists the supported sources.  
`cdx-cli search <SOURCE> --help` shows the available flags for that source plus
a brief example request and the relevant filter formats.

## Supported Search Sources

- `ALL` - exploratory search across all data sources
- `COMMENT` - legal commentaries
- `CR` - Czech legislation
- `ES` - EU court decisions
- `EU` - EU legislation
- `JD` - Czech case law
- `LT` - legal literature
- `SK` - Slovak legislation
- `VS` - contract templates

## Notes

- `ALL` is useful for orientation, not for final authoritative retrieval.
- `cdx-cli` currently focuses on search. The raw `cdx` curl wrapper can remain
  available separately for lower-level API work.
