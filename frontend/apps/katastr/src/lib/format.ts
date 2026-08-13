import * as shared from '@workspace/ui/lib/format'
import { format, parseISO } from 'date-fns'
import i18n from '@/lib/i18n'

// The date logic lives in @workspace/ui; these bind it to this app's language.

export function formatDate(isoString: string): string {
  return shared.formatDate(isoString, i18n.language)
}

/**
 * Time of day only, e.g. "09:12". Always a two-digit hour, unlike the locale's
 * own short time: the Czech format drops the leading zero, so "8:22" and "11:57"
 * put their colons in different places and a column of times reads ragged.
 */
export function formatTime(isoString: string): string {
  return format(parseISO(isoString), 'HH:mm')
}

/**
 * True when a ČÚZK timestamp carries a time of day. `datumProvedeni` does, down
 * to the second, but the field is typed as a plain date, so a date-only value
 * must not render as a fabricated midnight.
 */
export function hasTime(isoString: string): boolean {
  return isoString.includes('T')
}

export function formatDateTime(isoString: string): string {
  return shared.formatDateTime(isoString, i18n.language)
}

export function formatRelative(isoString: string | null | undefined): string {
  return shared.formatRelative(isoString, i18n.language)
}

export { errorMessage } from '@workspace/ui/lib/format'
