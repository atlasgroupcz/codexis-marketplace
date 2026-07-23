//  @ts-check

import { tanstackConfig } from '@tanstack/eslint-config'
import { config } from './react-internal.js'

// Patterns to ignore (third-party code, generated files, build output)
const ignorePatterns = [
  '**/*.generated.*',
  '**/node_modules/**',
  '**/dist/**',
  '**/build/**',
  '**/.next/**',
]

// Add ignores to each config object that has a files pattern
// This is necessary because in ESLint flat config, global ignores don't override explicit files patterns
const addIgnoresToConfigs = (configs) =>
  configs.map((cfg) =>
    cfg.files ? { ...cfg, ignores: [...(cfg.ignores || []), ...ignorePatterns] } : cfg
  )

export default [
  { ignores: ignorePatterns },
  ...addIgnoresToConfigs(tanstackConfig),
  ...addIgnoresToConfigs(config),
]
