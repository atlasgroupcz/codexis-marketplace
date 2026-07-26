import * as shared from '@workspace/ui/lib/format'
import i18n from '@/lib/i18n'

// The date logic lives in @workspace/ui; these bind it to this app's language.

export function formatDate(isoString: string): string {
  return shared.formatDate(isoString, i18n.language)
}

export function formatDateTime(isoString: string): string {
  return shared.formatDateTime(isoString, i18n.language)
}

export function formatRelative(isoString: string): string {
  return shared.formatRelative(isoString, i18n.language)
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
