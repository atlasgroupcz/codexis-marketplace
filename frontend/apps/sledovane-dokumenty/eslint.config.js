//  @ts-check

import { tanstackConfig } from '@tanstack/eslint-config'
import { config } from '@workspace/eslint-config/react-internal'

const ignorePatterns = [
  '**/node_modules/**',
  '**/dist/**',
  '**/build/**',
]

const addIgnoresToConfigs = (configs) =>
  configs.map((cfg) =>
    cfg.files ? { ...cfg, ignores: [...(cfg.ignores || []), ...ignorePatterns] } : cfg
  )

export default [
  { ignores: ignorePatterns },
  ...addIgnoresToConfigs(tanstackConfig),
  ...addIgnoresToConfigs(config),
]
