import { describe, expect, it } from 'vitest'
import { getDateFnsLocale } from '@/lib/format'

describe('getDateFnsLocale', () => {
  it('returns enUS for en', () => {
    expect(getDateFnsLocale('en').code).toBe('en-US')
  })

  it('returns cs for cs-CZ', () => {
    expect(getDateFnsLocale('cs-CZ').code).toBe('cs')
  })

  it('falls back to enUS for unknown language', () => {
    expect(getDateFnsLocale('de').code).toBe('en-US')
  })
})
