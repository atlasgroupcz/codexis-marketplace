import i18next from 'i18next'
import { initReactI18next } from 'react-i18next'
import en from '../../public/locales/en/translation.json'
import cs from '../../public/locales/cs/translation.json'
import sk from '../../public/locales/sk/translation.json'

const testI18n = i18next.createInstance()

void testI18n.use(initReactI18next).init({
  lng: 'en',
  fallbackLng: 'en',
  supportedLngs: ['en', 'cs', 'sk'],
  resources: {
    en: { translation: en },
    cs: { translation: cs },
    sk: { translation: sk },
  },
  interpolation: {
    escapeValue: false,
  },
  react: {
    useSuspense: false,
  },
  initImmediate: false,
})

export default testI18n
