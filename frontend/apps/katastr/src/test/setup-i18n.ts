// Tests render components directly, so nothing imports the app entrypoint and
// the i18n instance `main.tsx` sets up never initialises. `useTranslation()`
// then warns NO_I18NEXT_INSTANCE on every render and falls back to raw keys.
//
// Importing the app's own module here registers it as react-i18next's default
// (createI18n does `.use(initReactI18next)`), so components under test see the
// same translations they see in the browser.
import '@/lib/i18n'
