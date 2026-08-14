import type { Locale } from 'date-fns'
import { format, formatDistanceToNow, parseISO } from 'date-fns'
import { cs, enUS, sk } from 'date-fns/locale'

// Every app formatted dates through its own copy of this file. The copies had
// drifted - different fallback locales, different nullability - so the shared
// version takes the language as an argument and each app binds it to its own
// i18n instance in a two-line wrapper.

const dateFnsLocales: Record<string, Locale> = { en: enUS, cs, sk }

/** Unknown languages read English, the same fallback i18next itself uses. */
export function getDateFnsLocale(language: string): Locale {
  const baseLanguage = language.toLowerCase().split('-')[0] ?? ''
  return dateFnsLocales[baseLanguage] ?? enUS
}

/** Localized date, e.g. "26. 07. 2026". Empty input reads as a dash. */
export function formatDate(isoString: string | null | undefined, language: string): string {
  if (!isoString) return '-'
  return format(parseISO(isoString), 'P', { locale: getDateFnsLocale(language) })
}

/** Localized date and time. Empty input reads as a dash. */
export function formatDateTime(isoString: string | null | undefined, language: string): string {
  if (!isoString) return '-'
  return format(parseISO(isoString), 'Pp', { locale: getDateFnsLocale(language) })
}

/**
 * Time of day only, e.g. "09:12". Always a two-digit hour, unlike the locale's
 * own short time: the Czech format drops the leading zero, so "8:22" and "11:57"
 * put their colons in different places and a column of times reads ragged. That
 * is why this one takes no language.
 */
export function formatTime(isoString: string): string {
  return format(parseISO(isoString), 'HH:mm')
}

/**
 * True when a timestamp carries a time of day. ČÚZK's `datumProvedeni` does,
 * down to the second, but the field is typed as a plain date, so a date-only
 * value must not render as a fabricated midnight.
 */
export function hasTime(isoString: string): boolean {
  return isoString.includes('T')
}

/**
 * Relative time, e.g. "před 2 hodinami" / "2 hours ago". Empty input reads as a
 * dash, and an unparseable timestamp is returned as it came rather than throwing
 * into a render.
 */
export function formatRelative(isoString: string | null | undefined, language: string): string {
  if (!isoString) return '-'
  try {
    return formatDistanceToNow(parseISO(isoString), {
      addSuffix: true,
      locale: getDateFnsLocale(language),
    })
  } catch {
    return isoString
  }
}

/** Coerce an unknown thrown value into a human-readable message. */
export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e)
}
