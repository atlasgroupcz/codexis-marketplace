import { format, formatDistanceToNow, parseISO } from 'date-fns'
import { cs, enUS, sk } from 'date-fns/locale'
import type { Locale } from 'date-fns'
import i18n from '@/lib/i18n'

const dateFnsLocales: Record<string, Locale> = {
  en: enUS,
  cs,
  sk,
}

export function getDateFnsLocale(language: string): Locale {
  const baseLanguage = language.toLowerCase().split('-')[0]
  return dateFnsLocales[baseLanguage] ?? enUS
}

export function formatDate(isoString: string): string {
  return format(parseISO(isoString), 'P', {
    locale: getDateFnsLocale(i18n.language),
  })
}

export function formatDateTime(isoString: string): string {
  return format(parseISO(isoString), 'Pp', {
    locale: getDateFnsLocale(i18n.language),
  })
}

/** Relative "time since" (e.g. "před 3 dny"), for feed timestamps. */
export function formatRelative(isoString: string): string {
  return formatDistanceToNow(parseISO(isoString), {
    addSuffix: true,
    locale: getDateFnsLocale(i18n.language),
  })
}

/** First line of a markdown blob as plain text, for compact feed previews. */
export function plainSnippet(markdown: string, max = 160): string {
  const firstLine =
    markdown
      .split('\n')
      .map((line) => line.trim())
      .find((line) => line.length > 0) ?? ''
  const plain = firstLine
    .replace(/!?\[([^\]]*)\]\([^)]*\)/g, '$1') // links / images → label
    .replace(/[*_`#>~]/g, '') // strip md emphasis / heading / quote markers
    .replace(/\s+/g, ' ')
    .trim()
  return plain.length > max ? `${plain.slice(0, max).trimEnd()}…` : plain
}
