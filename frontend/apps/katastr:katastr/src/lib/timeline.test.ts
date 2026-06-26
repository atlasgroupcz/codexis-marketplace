import { describe, expect, it } from 'vitest'
import { deriveTimeline, latestMilestone } from './timeline'

const op = (nazev: string, datumProvedeni: string) => ({ nazev, datumProvedeni })

describe('deriveTimeline — event list', () => {
  it('builds one node per operation, oldest first, last is current', () => {
    const nodes = deriveTimeline({
      stav: 'Provedení záznamu',
      datum_prijeti: '2026-06-01',
      provedene_operace: [
        op('Provedení záznamu', '2026-06-05'),
        op('Založení řízení', '2026-06-01'),
      ],
    })
    expect(nodes).toHaveLength(2)
    expect(nodes[0].date).toBe('2026-06-01')
    expect(nodes[0].state).toBe('done')
    expect(nodes[1].state).toBe('current')
  })

  it('never fabricates future steps', () => {
    const nodes = deriveTimeline({
      stav: 'Plomba vyznačena',
      datum_prijeti: '2026-06-01',
      provedene_operace: [op('Přijetí návrhu', '2026-06-01'), op('Zaplombování', '2026-06-01')],
    })
    // exactly the two real events — no povolení/provedení/vyrozumění invented
    expect(nodes).toHaveLength(2)
  })

  it('keeps distinct ČÚZK operations even if both are plomba-related', () => {
    const nodes = deriveTimeline({
      stav: 'Povolení vkladu',
      datum_prijeti: '2026-04-24',
      provedene_operace: [
        op('Zaplombování', '2026-04-24'),
        op('Vyznačení plomby', '2026-04-24'),
        op('Povolení vkladu', '2026-05-15'),
      ],
    })
    expect(nodes).toHaveLength(3)
    expect(nodes.filter((n) => n.isPlomba)).toHaveLength(2)
    expect(nodes[0].rawName).toBe('Zaplombování')
    expect(nodes[1].rawName).toBe('Vyznačení plomby')
  })

  it('collapses only genuinely identical consecutive operations', () => {
    const nodes = deriveTimeline({
      stav: 'Plomba',
      datum_prijeti: '2026-04-24',
      provedene_operace: [
        op('Vyznačení plomby', '2026-04-24'),
        op('Vyznačení plomby', '2026-04-24'),
      ],
    })
    expect(nodes).toHaveLength(1)
  })

  it('shows a single node from stav when there are no operations', () => {
    const nodes = deriveTimeline({
      stav: 'Probíhá',
      datum_prijeti: '2026-06-01',
      provedene_operace: [],
    })
    expect(nodes).toHaveLength(1)
    expect(nodes[0].state).toBe('current')
    expect(nodes[0].rawName).toBe('Probíhá')
    expect(nodes[0].date).toBe('2026-06-01')
  })
})

describe('deriveTimeline — raw ČÚZK names', () => {
  it('uses the operation name exactly as ČÚZK reports it', () => {
    const n = deriveTimeline({
      stav: 'x',
      datum_prijeti: '2026-06-01',
      provedene_operace: [op('Vyrozumění o provedení vkladu', '2026-06-09')],
    })[0]
    expect(n.nameKey).toBeNull()
    expect(n.rawName).toBe('Vyrozumění o provedení vkladu')
  })

  it('flags plomba operations so they can be coloured', () => {
    const n = deriveTimeline({
      stav: 'x',
      datum_prijeti: '2026-06-01',
      provedene_operace: [op('Zaplombování', '2026-06-01')],
    })[0]
    expect(n.isPlomba).toBe(true)
  })
})

describe('deriveTimeline — negative outcome', () => {
  it('appends a terminal node for a rejected proceeding, ops stay done', () => {
    const nodes = deriveTimeline({
      stav: 'Zamítnuto',
      datum_prijeti: '2026-06-01',
      provedene_operace: [op('Přijetí návrhu', '2026-06-01'), op('Zaplombování', '2026-06-02')],
    })
    const last = nodes[nodes.length - 1]
    expect(last.isTerminal).toBe(true)
    expect(last.isNegative).toBe(true)
    expect(last.state).toBe('current')
    expect(nodes.filter((n) => n.state === 'done')).toHaveLength(2)
  })

  it('a benign ending is not marked negative', () => {
    const nodes = deriveTimeline({
      stav: 'Ukončení řízení',
      datum_prijeti: '2026-06-01',
      provedene_operace: [op('Založení řízení', '2026-06-01'), op('Ukončení řízení', '2026-06-03')],
    })
    expect(nodes.every((n) => !n.isNegative)).toBe(true)
    expect(nodes[nodes.length - 1].state).toBe('current')
  })
})

describe('latestMilestone', () => {
  it('returns the most recent operation', () => {
    const m = latestMilestone({
      stav: 'Povolení vkladu',
      provedene_operace: [op('Přijetí návrhu', '2026-06-01'), op('Povolení vkladu', '2026-06-12')],
    })
    expect(m.name).toBe('Povolení vkladu')
    expect(m.date).toBe('2026-06-12')
  })

  it('falls back to stav when there are no operations', () => {
    const m = latestMilestone({ stav: 'Podáno', provedene_operace: [] })
    expect(m.name).toBe('Podáno')
    expect(m.date).toBeNull()
  })
})
