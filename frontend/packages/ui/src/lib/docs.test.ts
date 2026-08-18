import { describe, expect, it } from 'vitest'
import { docsUrl } from './docs'

describe('docsUrl', () => {
  it('leaves the Czech manual unprefixed', () => {
    expect(docsUrl('sledovane-dokumenty', 'cs')).toBe(
      'https://docs.codexis.ai/codexis-ai-2-0/manual/aplikace/sledovane-dokumenty/',
    )
  })

  it('prefixes the other languages', () => {
    expect(docsUrl('katastr', 'en')).toBe(
      'https://docs.codexis.ai/en/codexis-ai-2-0/manual/aplikace/katastr-sledovana-rizeni/',
    )
    expect(docsUrl('katastr', 'sk')).toBe(
      'https://docs.codexis.ai/sk/codexis-ai-2-0/manual/aplikace/katastr-sledovana-rizeni/',
    )
  })

  it('reads a region tag as its language', () => {
    // i18next hands out detected tags like 'cs-CZ', not just 'cs'.
    expect(docsUrl('insolvence', 'cs-CZ')).toBe(
      'https://docs.codexis.ai/codexis-ai-2-0/manual/aplikace/hlidac-insolvenci/',
    )
  })

  it('falls back to English for a language the manual does not have', () => {
    expect(docsUrl('sid', 'de')).toBe(
      'https://docs.codexis.ai/en/codexis-ai-2-0/manual/aplikace/sid/',
    )
  })
})
