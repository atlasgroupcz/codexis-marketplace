# cdx-link-rewriter

Reads HTML from stdin, rewrites `cdx://` links to real URLs, writes to stdout.

Used as an onRender hook by the codexis plugin to transform `cdx://` scheme
links in chat messages before they reach the frontend.

## Requirements

- `CODEXIS_PLUGIN_BASE_URL` must be set in the environment (provided by the host via the plugin manifest's `env` block)

Example:

```bash
export CODEXIS_PLUGIN_BASE_URL="https://next.codexis.cz"
```

## URL rewriting

Only `href="cdx://..."` attributes are rewritten. The `cdx://` scheme in
plain text content is left unchanged.

Rules:
- trailing slash on `CODEXIS_PLUGIN_BASE_URL` is removed
- leading slash after `cdx://` is removed (avoids double slash)
- `cdx://` with empty path maps to `{CODEXIS_PLUGIN_BASE_URL}/`

Examples:
- `href="cdx://doc/CR10_2025_01_01"` -> `href="https://next.codexis.cz/doc/CR10_2025_01_01"`
- `href="cdx://"` -> `href="https://next.codexis.cz/"`

## Usage

```bash
echo '<a href="cdx://doc/CR10">doc</a>' | cdx-link-rewriter
# Output: <a href="https://next.codexis.cz/doc/CR10">doc</a>
```

## Build

```bash
./build.sh
```

## Integration with cdx-daemon

This binary is shipped inside the codexis plugin and declared as an onRender
hook in `hooks/hooks.json`. The plugin install/update lifecycle installs the
binary to `${HOME}/.local/bin`, and the backend resolves it from `PATH`:

```json
{
  "hooks": {
    "onRender": [
      {
        "command": "cdx-link-rewriter",
        "timeout": 5,
        "description": "Rewrites cdx:// links to absolute URLs"
      }
    ]
  }
}
```

The backend pipes completed message HTML through this binary via stdin/stdout
before returning it to the frontend.
