# @workspace/vitest-config

Shared Vitest settings for the plugin component UIs.

## Why this exists

Vitest externalizes dependencies for its SSR transform by default. `zod`'s ESM
entry then resolves to an empty module, so `z.object` is `undefined` at import
time and every schema-touching test fails to *collect* - not fail, collect. It
depends on the installed `node_modules` layout, so it passes on one machine and
fails on another.

The fix is one line - `server.deps.inline: ['zod']` - but it used to be copied
into four configs and missing from three others. This package holds it once.

## Usage

Dedicated Vitest configs (`vitest.config.ts`) - take the whole base and layer
your setup file / coverage on top:

```ts
import { baseConfig } from '@workspace/vitest-config/base'
import { defineConfig, mergeConfig } from 'vitest/config'

export default mergeConfig(
  baseConfig,
  defineConfig({ test: { setupFiles: ['./src/test/setup.ts'] } }),
)
```

Vite app configs (`vite.config.ts`) that carry a small `test:` block - take just
the inline list:

```ts
import { inlineDeps } from '@workspace/vitest-config/base'

// inside defineConfig(...)
test: { globals: true, server: { deps: { inline: inlineDeps } } }
```
