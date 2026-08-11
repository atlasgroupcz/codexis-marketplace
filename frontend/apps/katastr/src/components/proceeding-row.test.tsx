// @vitest-environment jsdom

import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { ProceedingSummary } from '@/lib/schemas'
import { ProceedingRow } from './proceeding-row'

// react-i18next returns the key by default in tests (no provider) - assert on keys/values we control.
const proc: ProceedingSummary = {
  cislo_rizeni: 'V-1234/2026-701',
  typ_rizeni: 'Vklad',
  label: 'Byt Brno-střed',
  stav: 'Povolení vkladu',
  stav_uhrady: 'U',
  stav_uhrady_label: 'Uhrazeno',
  datum_prijeti: '2026-06-01',
  added_on: '2026-06-01',
  last_check_at: '2026-06-12',
  last_op_date: '2026-06-12',
  operace_count: 3,
  changes_count: 1,
  unconfirmed_count: 2,
}

describe('ProceedingRow', () => {
  it('shows číslo, label, stav badge; no úhrada/milestone/date meta', () => {
    render(<ProceedingRow proceeding={proc} onClick={() => {}} selected={false} />)
    expect(screen.getByText('V-1234/2026-701')).toBeTruthy()
    expect(screen.getByText(/Byt Brno-střed/)).toBeTruthy()
    // stav appears
    expect(screen.getAllByText(/Povolení vkladu/).length).toBeGreaterThan(0)
    // decluttered: no "Uhrazeno", no "Poslední milník", no operation dump
    expect(screen.queryByText(/Uhrazeno/)).toBeNull()
    expect(screen.queryByText(/Vyznačení plomby/)).toBeNull()
    expect(screen.queryByText('Přijetí návrhu')).toBeNull()
  })

  it('shows the unread dot when there are unconfirmed changes', () => {
    render(<ProceedingRow proceeding={proc} onClick={() => {}} selected={false} />)
    expect(screen.getByTestId('unread-dot')).toBeTruthy()
  })

  it('shows the label inline when present', () => {
    render(<ProceedingRow proceeding={proc} onClick={() => {}} selected={false} />)
    expect(screen.getByTestId('proceeding-label')).toBeTruthy()
  })

  it('omits the label (no reserved empty line) when there is none', () => {
    render(
      <ProceedingRow proceeding={{ ...proc, label: '' }} onClick={() => {}} selected={false} />,
    )
    expect(screen.queryByTestId('proceeding-label')).toBeNull()
  })
})
