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
const CLOSED_TERMINAL_STEMS = ['ukonč', 'ukonc', 'uzavřen', 'uzavren', 'myln']

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
