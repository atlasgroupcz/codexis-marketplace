# cdx CLI wrapper

`cdx` is a thin wrapper around `curl`.

It adds two behaviors:
- rewrites `cdx://...` URLs to CODEXIS CDX API URLs
- adds Authorization header from required configuration

## Requirements

- `curl` must be available in `PATH`
- `CODEXIS_API_URL` must be set
- `CDX_API_JWT_AUTH` must be set

Example:

```bash
export CODEXIS_API_URL="https://app.codexis.cz"
export CDX_API_JWT_AUTH="Bearer <jwt>"
```

`CDX_API_JWT_AUTH` may also be set to the raw JWT value (`<jwt>`); `cdx` adds
the `Bearer` prefix when needed.

If one or both variables are missing from the process environment, `cdx` tries
to read them from `~/.cdx/.env`. Process environment values win over values in
the file.

## URL rewriting

Any argument starting with `cdx://` is rewritten to:

```text
{CODEXIS_API_URL}/rest/cdx-api/{path}
```

Rules:
- trailing slash on `CODEXIS_API_URL` is removed
- leading slash after `cdx://` is removed
- `cdx://` with empty path maps to `{CODEXIS_API_URL}/rest/cdx-api`

Examples:
- `cdx://doc/CR10_2025_01_01` -> `https://app.codexis.cz/rest/cdx-api/doc/CR10_2025_01_01`
- `cdx://` -> `https://app.codexis.cz/rest/cdx-api`

All other arguments are passed to `curl` unchanged.

## Authentication via `CDX_API_JWT_AUTH`

`CDX_API_JWT_AUTH` is required. `cdx` reads it from the process environment or
falls back to `~/.cdx/.env`, then adds `-H <Authorization header>`.

Accepted formats:
- `Authorization: Bearer <jwt>` (used as-is)
- `Bearer <jwt>` (converted to `Authorization: Bearer <jwt>`)
- `<jwt>` (JWT-like `a.b.c` heuristic, converted to `Authorization: Bearer <jwt>`)
- any other value (converted to `Authorization: <value>`)

If `CDX_API_JWT_AUTH` is missing or empty in both places, `cdx` exits with an
error.

## Usage

```bash
cdx cdx://doc/CR10_2025_01_01
cdx -sS -X POST cdx://search -H 'Content-Type: application/json' -d '{"q":"test"}'
```

## Integration with `cdx-daemon`

When shell commands are executed through `cdx-daemon`, it can obtain a CDX API JWT from CODEXIS admin endpoint and inject `CDX_API_JWT_AUTH` into the command environment.

This allows `cdx` to work without manually exporting JWT in user shell sessions.

## Helper script: `get-jwt.sh`

`utils/cdx/get-jwt.sh` reads `application-prod.properties`, resolves Spring-style
`${ENV:default}` placeholders for:
- `codexis.base.url`
- `codexis.admin.user`
- `codexis.admin.secret`
- `codexis.jwt.subject`
- `codexis.token.timeout`

Then it calls `POST /rest/admin/cdx-api/token` and prints only the JWT token to stdout.

Examples:

```bash
# default properties file: ../../application-prod.properties
./utils/cdx/get-jwt.sh

# explicit properties file path
./utils/cdx/get-jwt.sh /path/to/application-prod.properties
```
