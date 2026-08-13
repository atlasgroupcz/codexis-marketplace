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

  it('shows the time of day, so same-day steps do not read as simultaneous', () => {
    render(
      <ProceedingTimeline
        proc={{
          ...base,
          stav: 'Probíhá',
          provedene_operace: [
            { nazev: 'Založení řízení', datumProvedeni: '2026-01-15T07:42:54' },
            { nazev: 'Informace o vyznačení plomby', datumProvedeni: '2026-01-15T07:54:18' },
          ],
        }}
      />,
    )
    expect(screen.getByTestId('timeline-node-op:0').textContent).toContain('07:42')
    expect(screen.getByTestId('timeline-node-op:1').textContent).toContain('07:54')
  })

  it('omits the time when ČÚZK sends a bare date, rather than inventing midnight', () => {
    render(
      <ProceedingTimeline
        proc={{
          ...base,
          stav: 'Probíhá',
          provedene_operace: [{ nazev: 'Založení řízení', datumProvedeni: '2026-01-15' }],
        }}
      />,
    )
    expect(screen.getByTestId('timeline-node-op:0').textContent).not.toContain('00:00')
  })

  it('swaps the now chip for a closed one once ČÚZK reports the ending', () => {
    render(
      <ProceedingTimeline
        proc={{
          ...base,
          stav: 'Ukončeno',
          provedene_operace: [
            { nazev: 'Vyrozumění o provedení vkladu', datumProvedeni: '2026-06-09' },
            { nazev: 'Ukončení řízení', datumProvedeni: '2026-06-12' },
          ],
        }}
      />,
    )
    expect(screen.getByTestId('timeline-node-op:1').getAttribute('data-state')).toBe('done')
    expect(screen.queryByText('now')).toBeNull()
    expect(screen.getByText('closed')).toBeTruthy()
  })

  it('marks a rejected proceeding with its own chip, not the closed one', () => {
    render(
      <ProceedingTimeline
        proc={{
          ...base,
          stav: 'Zamítnuto',
          provedene_operace: [{ nazev: 'Zaplombování', datumProvedeni: '2026-06-01' }],
        }}
      />,
    )
    expect(screen.getByText('rejected')).toBeTruthy()
    expect(screen.queryByText('closed')).toBeNull()
    expect(screen.queryByText('now')).toBeNull()
  })

  it('an unrecognised ending keeps the now chip rather than inventing one', () => {
    render(
      <ProceedingTimeline
        proc={{
          ...base,
          stav: 'Něco, co ČÚZK vymyslel dnes ráno',
          provedene_operace: [{ nazev: 'Založení řízení', datumProvedeni: '2026-06-01' }],
        }}
      />,
    )
    expect(screen.getByText('now')).toBeTruthy()
    expect(screen.queryByText('closed')).toBeNull()
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
