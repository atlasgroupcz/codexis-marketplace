// Semantic reading of a ČÚZK `stav` string, shared by the badge (colour) and the
// timeline (terminal outcome) so the two panes can't disagree about what a status
// means - a rejected proceeding must read as "bad" in both, not red in the
// timeline and a neutral chip in the badge.

// Negative/closing outcomes - a proceeding that ended without an entry
// (rejected / stopped / withdrawn / dismissed).
const NEGATIVE_TERMINAL_STEMS = ['zamít', 'zastav', 'zpětvz', 'zpetvz', 'odmít', 'vzat zpět']

// Endings that carry no blame - the proceeding simply closed ("Ukončeno",
// "Řízení bylo uzavřeno jako mylné"). ČÚZK writes `stav` as free text with no
// enumeration behind it, so this list can never be complete. It only decides
// whether the last node keeps the "teď" chip; an unrecognised ending leaves the
// chip on a finished proceeding and nothing else.
const MYLNE_STEM = 'myln'
const CLOSED_TERMINAL_STEMS = ['ukonč', 'ukonc', 'uzavřen', 'uzavren', MYLNE_STEM]

export function isNegativeTerminal(stav: string): boolean {
  const s = stav.toLowerCase()
  return NEGATIVE_TERMINAL_STEMS.some((stem) => s.includes(stem))
}

/** Any ending, good or bad - the proceeding is no longer running. */
export function isTerminal(stav: string): boolean {
  const s = stav.toLowerCase()
  return isNegativeTerminal(stav) || CLOSED_TERMINAL_STEMS.some((stem) => s.includes(stem))
}

export function isPlomba(text: string): boolean {
  return text.toLowerCase().includes('plomb')
}

/**
 * Which of the states we recognise this is, or null for one we do not.
 *
 * The badge needs a colour and a short name for the same string, and deciding
 * them from two keyword lists let them disagree: "Řízení bylo uzavřeno jako
 * mylné" read as an ending in one and as unrecognised in the other. One list
 * answers both.
 *
 * Order matters. A bad ending wins over every other word in the sentence; a
 * mylné řízení is worded as an ending too, so it is claimed before the general
 * one; and "Vyrozumění o provedení vkladu" contains "proveden", so notified
 * comes before registered.
 */
export type StavKind =
  | 'negative' // ended without an entry
  | 'mistaken' // the proceeding should never have been opened
  | 'closed' // ended, no blame
  | 'notified' // participants told the entry was made
  | 'registered' // the entry itself was made
  | 'allowed' // the entry was approved, not yet made
  | 'plomba'
  | 'running'
  | 'received' // the application arrived

export function stavKind(stav: string): StavKind | null {
  const s = stav.toLowerCase()
  if (isNegativeTerminal(stav)) return 'negative'
  if (s.includes(MYLNE_STEM)) return 'mistaken'
  if (isTerminal(stav)) return 'closed'
  if (s.includes('vyrozum')) return 'notified'
  if (s.includes('proveden')) return 'registered'
  if (s.includes('povolen')) return 'allowed'
  if (isPlomba(stav)) return 'plomba'
  if (s.includes('probíhá')) return 'running'
  if (s.includes('přijet') || s.includes('podáno')) return 'received'
  return null
}
