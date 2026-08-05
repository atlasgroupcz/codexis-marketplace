import { describe, expect, it } from 'vitest'
import { formatDate, formatRelative, getDateFnsLocale } from './format'

describe('getDateFnsLocale', () => {
  it('returns enUS for en', () => {
    expect(getDateFnsLocale('en').code).toBe('en-US')
  })

  it('returns cs for cs-CZ', () => {
    expect(getDateFnsLocale('cs-CZ').code).toBe('cs')
  })

  it('falls back to enUS for an unknown language', () => {
    expect(getDateFnsLocale('de').code).toBe('en-US')
  })
})

describe('formatDate', () => {
  it('formats in the language it is given', () => {
    expect(formatDate('2026-07-26T10:00:00Z', 'cs')).toBe('26.07.2026')
    expect(formatDate('2026-07-26T10:00:00Z', 'en')).toBe('07/26/2026')
  })

  it('reads a missing date as a dash instead of throwing into a render', () => {
    expect(formatDate(null, 'cs')).toBe('-')
  })
})

describe('formatRelative', () => {
  it('returns an unparseable timestamp as it came', () => {
    expect(formatRelative('not-a-date', 'cs')).toBe('not-a-date')
  })

  it('reads a missing timestamp as a dash', () => {
    expect(formatRelative(undefined, 'cs')).toBe('-')
  })
})
