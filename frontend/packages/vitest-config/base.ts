import react from '@vitejs/plugin-react'
import type { ViteUserConfig } from 'vitest/config'

// The single source of truth for which dependencies Vitest must inline.
//
// Vitest externalizes deps for its SSR transform by default, which leaves zod's
// ESM entry resolving to an empty module - `z.object` is undefined at import
// time and every schema-touching test fails to collect. Whether it bites
// depends on the installed node_modules layout, so it passes on one machine and
// fails on another; inlining makes it deterministic. Kept here, once, so the
// four consuming configs can no longer drift out of sync.
export const inlineDeps = ['zod']

// Vitest settings shared by the plugin component UIs. Consumers layer their own
// setupFiles, coverage and timeouts on top via `mergeConfig`.
export const baseConfig = {
  plugins: [react()],
  // Vite 8 resolves tsconfig `paths` natively; the vite-tsconfig-paths plugin
  // that used to do this now warns on every run that it is redundant.
  resolve: { tsconfigPaths: true },
  test: {
    environment: 'jsdom',
    globals: true,
    server: { deps: { inline: inlineDeps } },
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    exclude: ['node_modules', 'dist', '.turbo'],
    // A component UI with no test files yet is a green run, not a CI failure -
    // consumers of this base config don't each need the --passWithNoTests flag.
    passWithNoTests: true,
  },
} satisfies ViteUserConfig
