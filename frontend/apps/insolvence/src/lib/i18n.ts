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
    fallbackLng: 'cs',
    supportedLngs: ['en', 'cs', 'sk'],
    defaultNS: 'translation',
    // DEV is true under Vitest too, and i18next then dumps its whole config
    // into the test log on every init - keep the debug output for real dev only.
    debug: import.meta.env.DEV && !import.meta.env.VITEST,
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
  })

export default i18n
