import type { i18n, Resource } from 'i18next'
import i18next from 'i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import { initReactI18next } from 'react-i18next'

export interface CreateI18nOptions {
  // `Resource` (from i18next) rather than the brief's `Record<string, { translation: unknown }>`:
  // `unknown` isn't assignable to i18next's `ResourceKey` (`string | { [key: string]: any }`),
  // so `i18next.init()` fails to typecheck. `Resource` is i18next's own type for this shape and
  // accepts the same `{ en: { translation: {...} }, ... }` literals unchanged.
  resources: Resource
  fallbackLng?: string
  supportedLngs?: Array<string>
}

export function createI18n({
  resources,
  fallbackLng = 'en',
  supportedLngs = ['en', 'cs', 'sk'],
}: CreateI18nOptions): i18n {
  const instance = i18next.createInstance()
  void instance
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
      resources,
      fallbackLng,
      supportedLngs,
      defaultNS: 'translation',
      interpolation: { escapeValue: false },
      detection: {
        order: ['querystring', 'localStorage', 'navigator'],
        lookupQuerystring: 'lang',
        caches: ['localStorage'],
      },
    })
  return instance
}
