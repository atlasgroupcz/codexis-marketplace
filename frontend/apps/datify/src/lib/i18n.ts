import { createI18n } from '@workspace/ui/lib/i18n'
import en from '../../public/locales/en/translation.json'
import cs from '../../public/locales/cs/translation.json'
import sk from '../../public/locales/sk/translation.json'

// Shared init (LanguageDetector + react-i18next, same fallback and supported
// languages, querystring `lang` detection). It returns its own instance rather
// than configuring the global i18next singleton, which the local copy did.
export default createI18n({
  resources: {
    en: { translation: en },
    cs: { translation: cs },
    sk: { translation: sk },
  },
})
