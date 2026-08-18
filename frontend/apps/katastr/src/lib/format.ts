import * as shared from '@workspace/ui/lib/format'
import i18n from '@/lib/i18n'

// The date logic lives in @workspace/ui; these bind it to this app's language.

export function formatDate(isoString: string): string {
  return shared.formatDate(isoString, i18n.language)
}

export function formatRelative(isoString: string | null | undefined): string {
  return shared.formatRelative(isoString, i18n.language)
}

// Language-free, so they need no binding and pass straight through.
export { errorMessage, formatTime, hasTime } from '@workspace/ui/lib/format'
