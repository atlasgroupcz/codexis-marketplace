import type { TFunction } from 'i18next'

/**
 * The single-letter ČÚZK payment code in the reader's language.
 *
 * The wire also carries a label, but it is built in Rust, which has no language
 * context, so it is Czech whoever reads it. The code is the language-free part
 * of the payload, and the header and the change log both translate it here so
 * the key prefix and the fallback cannot drift apart.
 *
 * ČÚZK's own code is the fallback, so an unknown one shows rather than blanks.
 */
export function uhradaLabel(t: TFunction, code: string | null | undefined): string {
  if (!code) return '---'
  return t(`proceedings.uhrada.${code}`, { defaultValue: code })
}
