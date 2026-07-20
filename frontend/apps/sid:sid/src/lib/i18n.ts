import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import en from '../../public/locales/en/translation.json'
import cs from '../../public/locales/cs/translation.json'
import sk from '../../public/locales/sk/translation.json'

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    fallbackLng: 'en',
    supportedLngs: ['en', 'cs', 'sk'],
    defaultNS: 'translation',
    debug: import.meta.env.DEV,
    interpolation: {
      escapeValue: false,
    },
    resources: {
      en: { translation: en },
      cs: { translation: cs },
      sk: { translation: sk },
    },
    detection: {
      order: ['querystring', 'localStorage', 'navigator'],
      lookupQuerystring: 'lang',
      lookupLocalStorage: 'i18nextLng',
      caches: ['localStorage'],
    },
    react: {
      useSuspense: true,
    },
    initImmediate: false,
  })

export default i18n
