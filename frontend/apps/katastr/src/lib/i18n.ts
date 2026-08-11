import { createI18n } from '@workspace/ui/lib/i18n'
import cs from '../../public/locales/cs/translation.json'
import en from '../../public/locales/en/translation.json'
import sk from '../../public/locales/sk/translation.json'

// Shared init (LanguageDetector + react-i18next, same fallback/supported langs,
// querystring `lang` detection). Consumed everywhere via this default export -
// Providers wraps it, format.ts reads its `.language`.
export default createI18n({
  resources: {
    en: { translation: en },
    cs: { translation: cs },
    sk: { translation: sk },
  },
})
