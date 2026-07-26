const DOCS_ORIGIN = 'https://docs.codexis.ai'

// Czech is the default language of the manual and carries no prefix.
const LANG_PREFIX: Record<string, string> = { cs: '', en: 'en/', sk: 'sk/' }

/**
 * Manual page of each app, keyed by app id. The slug does not follow from the app
 * name (insolvence -> hlidac-insolvenci), so it is spelled out here; an app with no
 * manual page is simply absent and cannot be linked by mistake.
 */
const DOCS_PATH = {
  'sledovane-dokumenty': 'sledovane-dokumenty',
  'sledovana-judikatura': 'sledovana-judikatura',
  insolvence: 'hlidac-insolvenci',
  katastr: 'katastr-sledovana-rizeni',
  'sbirka-listin': 'sbirka-listin',
  sid: 'sid',
} as const

export type DocsApp = keyof typeof DOCS_PATH

/** Manual page of `app` in the reader's language. Unknown languages read English. */
export function docsUrl(app: DocsApp, lang: string): string {
  const prefix = LANG_PREFIX[lang.split('-')[0] ?? ''] ?? LANG_PREFIX.en
  return `${DOCS_ORIGIN}/${prefix}codexis-ai-2-0/manual/aplikace/${DOCS_PATH[app]}/`
}
