import type { Listina } from './schemas'

/** The SL sequence number from a číslo listiny ("B 8573/SL95/MSPH" → 95). It
 * rises monotonically with each filing, so it's a reliable "how new" key
 * (unlike a lexicographic sort of the whole číslo, where SL10 < SL2). */
export function slNumber(cislo: string): number {
  const m = /SL0*(\d+)/i.exec(cislo)
  return m ? Number(m[1]) : 0
}

/** A subject's listiny, newest filing first. */
export function byNewest(listiny: Array<Listina>): Array<Listina> {
  return [...listiny].sort((a, b) => slNumber(b.cislo) - slNumber(a.cislo))
}
