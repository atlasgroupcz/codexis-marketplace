import { format, formatDistanceToNow, parseISO } from 'date-fns'
import { cs, enUS, sk } from 'date-fns/locale'
import type { Locale } from 'date-fns'
import i18n from '@/lib/i18n'

const dateFnsLocales: Record<string, Locale> = {
  en: enUS,
  cs,
  sk,
}

function getDateFnsLocale(language: string): Locale {
  const baseLanguage = language.toLowerCase().split('-')[0]
  return dateFnsLocales[baseLanguage] ?? cs
}

/** Format an ISO date (YYYY-MM-DD) as a localized date. Empty input → em dash. */
export function formatDate(isoString: string | null | undefined): string {
  if (!isoString) return '-'
  try {
    return format(parseISO(isoString), 'P', {
      locale: getDateFnsLocale(i18n.language),
    })
  } catch {
    return isoString
  }
}

/** Format an ISO datetime as a localized date + time. */
export function formatDateTime(isoString: string | null | undefined): string {
  if (!isoString) return '-'
  try {
    return format(parseISO(isoString), 'Pp', {
      locale: getDateFnsLocale(i18n.language),
    })
  } catch {
    return isoString
  }
}

/** Relative time, e.g. "před 5 minutami" / "5 minutes ago" / "pred 3 dňami". */
export function formatRelative(isoString: string | null | undefined): string {
  if (!isoString) return '-'
  try {
    return formatDistanceToNow(parseISO(isoString), {
      addSuffix: true,
      locale: getDateFnsLocale(i18n.language),
    })
  } catch {
    return isoString
  }
}
