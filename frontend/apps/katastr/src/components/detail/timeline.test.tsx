// @vitest-environment jsdom

import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { ProceedingDetail } from '@/lib/schemas'
import { ProceedingTimeline } from './timeline'

const base = {
  cislo_rizeni: 'V-1234/2026-701',
  typ_rizeni: 'Vklad',
  label: '',
  stav_uhrady: 'U',
  stav_uhrady_label: 'Uhrazeno',
  datum_prijeti: '2026-06-01',
  added_on: '2026-06-01',
  last_check_at: '2026-06-12',
  last_op_date: '2026-06-12',
  operace_count: 2,
  changes_count: 0,
  unconfirmed_count: 0,
  changes: [],
} as unknown as ProceedingDetail

describe('ProceedingTimeline', () => {
  it('renders one node per operation, last is current', () => {
    render(
      <ProceedingTimeline
        proc={{
          ...base,
          stav: 'Plomba vyznačena',
          provedene_operace: [
            { nazev: 'Přijetí návrhu', datumProvedeni: '2026-06-01' },
            { nazev: 'Zaplombování', datumProvedeni: '2026-06-01' },
          ],
        }}
      />,
    )
    expect(screen.getByTestId('timeline-node-op:0')).toBeTruthy()
    expect(screen.getByTestId('timeline-node-op:1')).toBeTruthy()
    expect(screen.getByTestId('timeline-node-op:1').getAttribute('data-state')).toBe('current')
    // no fabricated future steps
    expect(screen.queryByTestId('timeline-node-op:2')).toBeNull()
  })

  it('rejected proceeding renders a terminal node', () => {
    render(
      <ProceedingTimeline
        proc={{
          ...base,
          stav: 'Zamítnuto',
          provedene_operace: [{ nazev: 'Přijetí návrhu', datumProvedeni: '2026-06-01' }],
        }}
      />,
    )
    expect(screen.getByTestId('timeline-node-terminal')).toBeTruthy()
  })
})
