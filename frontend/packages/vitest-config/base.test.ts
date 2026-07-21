import { describe, expect, it } from 'vitest'

import { baseConfig, inlineDeps } from './base'

describe('vitest base preset', () => {
  it('inlines zod so schema-touching tests collect under the SSR transform', () => {
    // This is the single source of truth that used to be copy-pasted into four
    // configs. If it silently drops, every zod-schema test fails to collect —
    // the exact regression this package exists to prevent.
    expect(inlineDeps).toContain('zod')
    expect(baseConfig.test?.server?.deps?.inline).toContain('zod')
  })

  it('runs component tests in jsdom with globals enabled', () => {
    expect(baseConfig.test?.environment).toBe('jsdom')
    expect(baseConfig.test?.globals).toBe(true)
  })
})
