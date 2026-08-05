import { baseConfig } from '@workspace/vitest-config/base'
import { defineConfig, mergeConfig } from 'vitest/config'

// Shared test config (jsdom, globals, zod inline, passWithNoTests). sid used to
// carry these settings inside vite.config.ts, which is the one place the shared
// app config cannot type - vite's UserConfig has no `test`.
export default mergeConfig(baseConfig, defineConfig({}))
