import { baseConfig } from '@workspace/vitest-config/base'
import { defineConfig, mergeConfig } from 'vitest/config'

// Shared test config (jsdom, globals, zod inline, passWithNoTests). Kept as a
// thin extension so app-specific setup/coverage can be layered on later.
export default mergeConfig(baseConfig, defineConfig({}))
