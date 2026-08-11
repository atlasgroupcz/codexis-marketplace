import { createI18n } from '@workspace/ui/lib/i18n'
import cs from '../../public/locales/cs/translation.json'
import en from '../../public/locales/en/translation.json'
import sk from '../../public/locales/sk/translation.json'

// Shared bootstrap: fallback 'en', supported en/cs/sk, querystring→localStorage→
// navigator detection, escapeValue off, synchronous init - identical to the
// hand-rolled setup this replaced.
export default createI18n({
  resources: {
    en: { translation: en },
    cs: { translation: cs },
    sk: { translation: sk },
  },
})
