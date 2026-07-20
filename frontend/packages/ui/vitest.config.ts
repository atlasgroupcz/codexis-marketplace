import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tsconfigPaths from 'vite-tsconfig-paths'

export default defineConfig({
  plugins: [react(), tsconfigPaths()],
  test: {
    environment: 'jsdom',
    globals: true,
    // Vitest externalizes deps for its SSR transform by default, which leaves
    // zod's ESM entry resolving to an empty module — `z.object` is undefined at
    // import time and every schema-touching test fails to collect. Whether it
    // bites depends on the installed node_modules layout, so it passes on one
    // machine and fails on another; inlining makes it deterministic.
    server: { deps: { inline: ['zod'] } },
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    exclude: ['node_modules', 'dist'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      include: ['src/lib/**/*.ts', 'src/hooks/**/*.ts'],
      exclude: ['**/*.d.ts', '**/test/**'],
    },
  },
})
