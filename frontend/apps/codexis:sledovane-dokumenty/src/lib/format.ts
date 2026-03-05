import { format, parseISO } from 'date-fns'
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
