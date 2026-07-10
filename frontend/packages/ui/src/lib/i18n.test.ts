import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18n } from './i18n'

const resources = {
  en: { translation: { hi: 'Hello' } },
  cs: { translation: { hi: 'Ahoj' } },
  sk: { translation: { hi: 'Ahoj' } },
}

// jsdom's default navigator.language ('en-US') is itself a supported language,
// so i18next-browser-languagedetector's 'navigator' fallback would resolve 'en'
// before fallbackLng ever gets a chance to run, masking the behavior under test.
// Pin navigator to an unsupported locale to isolate fallbackLng from the host's
// OS/browser locale (production behavior of createI18n is unchanged).
beforeEach(() => {
  vi.stubGlobal('navigator', { ...navigator, language: 'xx-XX', languages: ['xx-XX'] })
})
afterEach(() => vi.unstubAllGlobals())

describe('createI18n', () => {
  it('resolves keys and honors fallbackLng', () => {
    const i18n = createI18n({ resources, fallbackLng: 'cs' })
    expect(i18n.t('hi')).toBe('Ahoj')
  })

  it('returns an isolated instance (two instances do not clash)', () => {
    const a = createI18n({ resources, fallbackLng: 'en' })
    const b = createI18n({ resources, fallbackLng: 'cs' })
    expect(a).not.toBe(b)
  })
})
