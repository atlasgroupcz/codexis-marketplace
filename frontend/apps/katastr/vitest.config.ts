import { baseConfig } from '@workspace/vitest-config/base'
import { defineConfig, mergeConfig } from 'vitest/config'

// Shared test config (jsdom, globals, zod inline, passWithNoTests), plus the
// i18n bootstrap the app does in main.tsx - tests bypass that entrypoint.
export default mergeConfig(
  baseConfig,
  defineConfig({
    test: { setupFiles: ['./src/test/setup-i18n.ts'] },
  }),
)
