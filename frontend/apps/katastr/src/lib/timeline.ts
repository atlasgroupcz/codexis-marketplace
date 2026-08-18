import { isNegativeTerminal, isPlomba, isTerminal } from '@/lib/stav'

export type TimelineState = 'done' | 'current'

export interface TimelineNode {
  key: string
  rawName: string | null // the operation/stav text exactly as ČÚZK reports it
  date: string | null // ISO date or null
  state: TimelineState
  isPlomba: boolean
  isTerminal: boolean // this is where the proceeding ended
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

/**
 * A flat, honest list of what actually happened - one node per real ČÚZK
 * operation, oldest first, named exactly as ČÚZK reports it. No speculative
 * future phases: a proceeding can be rejected or withdrawn, so we never imply
 * steps that may not come.
 */
export function deriveTimeline(proc: TimelineInput): Array<TimelineNode> {
  const ops = proc.provedene_operace

  const raw: Array<TimelineNode> = ops.map((op, i) => ({
    key: `op:${i}`,
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
    (n, i) => i === 0 || n.rawName !== raw[i - 1]?.rawName,
  )

  const negative = isNegativeTerminal(proc.stav)
  const terminal = isTerminal(proc.stav)

  if (negative) {
    // The bad outcome is a distinct closing node after everything that happened.
    // `done`, not `current`: a rejected proceeding has no step in progress. The
    // node used to claim both, and every reader had to know that `isTerminal`
    // wins over `state` to draw it right.
    nodes.push({
      key: 'terminal',
      rawName: proc.stav,
      date: null,
      state: 'done',
      isPlomba: false,
      isTerminal: true,
      isNegative: true,
    })
  } else {
    // A closed proceeding has no step happening "now" - the last operation is
    // where it ended. ČÚZK's own wording is the only hint we get, so a miss here
    // just swaps the closing chip back for "teď"; it never affects which nodes
    // exist or how they connect.
    const last = nodes[nodes.length - 1]
    if (last) {
      if (terminal) last.isTerminal = true
      else last.state = 'current'
    }
  }

  // No operations yet - show the current state as the single node.
  if (nodes.length === 0) {
    nodes.push({
      key: 'stav',
      rawName: proc.stav,
      date: proc.datum_prijeti || null,
      state: terminal ? 'done' : 'current',
      isPlomba: isPlomba(proc.stav),
      isTerminal: terminal,
      isNegative: false,
    })
  }

  return nodes
}
