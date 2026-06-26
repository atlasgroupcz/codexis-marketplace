export type TimelineState = 'done' | 'current'

export interface TimelineNode {
  key: string
  nameKey: string | null // reserved for known phases; currently always null (raw ČÚZK names)
  rawName: string | null // the operation/stav text exactly as ČÚZK reports it
  date: string | null // ISO date or null
  state: TimelineState
  isPlomba: boolean
  isTerminal: boolean // distinct closing node derived from `stav`, not an operation
  isNegative: boolean // ended badly (rejected/stopped/withdrawn)
}

interface Operace {
  nazev: string
  datumProvedeni: string
}

interface TimelineInput {
  stav: string
  datum_prijeti: string
  provedene_operace: Array<Operace>
}

// Negative/closing outcomes — a proceeding that ended without an entry.
const NEGATIVE_TERMINAL_STEMS = ['zamít', 'zastav', 'zpětvz', 'zpetvz', 'odmít', 'vzat zpět']

function isNegativeTerminal(stav: string): boolean {
  const s = stav.toLowerCase()
  return NEGATIVE_TERMINAL_STEMS.some((stem) => s.includes(stem))
}

function isPlomba(text: string): boolean {
  return text.toLowerCase().includes('plomb')
}

/**
 * A flat, honest list of what actually happened — one node per real ČÚZK
 * operation, oldest first, named exactly as ČÚZK reports it. No speculative
 * future phases: a proceeding can be rejected or withdrawn, so we never imply
 * steps that may not come.
 */
export function deriveTimeline(proc: TimelineInput): Array<TimelineNode> {
  const ops = [...proc.provedene_operace].sort((a, b) =>
    a.datumProvedeni < b.datumProvedeni ? -1 : a.datumProvedeni > b.datumProvedeni ? 1 : 0,
  )

  const raw: Array<TimelineNode> = ops.map((op, i) => ({
    key: `op:${i}`,
    nameKey: null,
    rawName: op.nazev,
    date: op.datumProvedeni,
    state: 'done',
    isPlomba: isPlomba(op.nazev),
    isTerminal: false,
    isNegative: false,
  }))

  // Collapse only genuinely identical consecutive operations (ČÚZK occasionally
  // reports the same step twice). Distinct names stay distinct.
  const nodes: Array<TimelineNode> = raw.filter(
    (n, i) => i === 0 || n.rawName !== raw[i - 1].rawName,
  )

  const negative = isNegativeTerminal(proc.stav)

  if (negative) {
    // The bad outcome is a distinct closing node after everything that happened.
    nodes.push({
      key: 'terminal',
      nameKey: null,
      rawName: proc.stav,
      date: null,
      state: 'current',
      isPlomba: false,
      isTerminal: true,
      isNegative: true,
    })
  } else if (nodes.length > 0) {
    nodes[nodes.length - 1].state = 'current'
  }

  // No operations yet — show the current state as the single node.
  if (nodes.length === 0) {
    nodes.push({
      key: 'stav',
      nameKey: null,
      rawName: proc.stav,
      date: proc.datum_prijeti || null,
      state: 'current',
      isPlomba: isPlomba(proc.stav),
      isTerminal: false,
      isNegative: false,
    })
  }

  return nodes
}

export function latestMilestone(proc: {
  stav: string
  last_op_date?: string | null
  provedene_operace: Array<Operace>
}): { name: string | null; nameKey: string | null; date: string | null } {
  const ops = proc.provedene_operace
  if (ops.length > 0) {
    const last = ops.reduce((a, b) => (a.datumProvedeni >= b.datumProvedeni ? a : b))
    return { name: last.nazev, nameKey: null, date: last.datumProvedeni }
  }
  return { name: proc.stav, nameKey: null, date: proc.last_op_date ?? null }
}
