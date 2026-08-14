// @vitest-environment jsdom

import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { OverviewResponse, ProceedingSummary } from '@/lib/schemas'
import { proceedingFixture as proc } from '@/test/fixtures'
import { ProceedingsList } from './proceedings-list'

function overview(proceedings: Array<ProceedingSummary>): OverviewResponse {
  return {
    mode: 'overview',
    generated_at: '2026-04-01T00:00:00Z',
    proceedings,
    api_key_configured: true,
    api_key_masked: '',
  }
}

const DATA = overview([
  proc({ cislo_rizeni: 'V-1/2026-701', unconfirmed_count: 2 }),
  proc({ cislo_rizeni: 'V-2/2026-701', unconfirmed_count: 0 }),
  proc({ cislo_rizeni: 'V-3/2026-701', unconfirmed_count: 1 }),
])

const noop = () => {}

describe('ProceedingsList filter', () => {
  it('shows all rows and the unconfirmed count when filter=all', () => {
    render(
      <ProceedingsList
        data={DATA}
        loading={false}
        error={null}
        refetch={noop}
        selectedCislo={null}
        onSelect={noop}
        filter="all"
        onFilterChange={noop}
      />,
    )
    expect(screen.getByTestId('proceeding-row-V-1/2026-701')).toBeTruthy()
    expect(screen.getByTestId('proceeding-row-V-2/2026-701')).toBeTruthy()
    expect(screen.getByTestId('proceeding-row-V-3/2026-701')).toBeTruthy()
    // count = proceedings with unconfirmed_count > 0
    expect(screen.getByTestId('filter-unconfirmed-count').textContent).toBe('2')
  })

  it('hides confirmed rows when filter=unconfirmed', () => {
    render(
      <ProceedingsList
        data={DATA}
        loading={false}
        error={null}
        refetch={noop}
        selectedCislo={null}
        onSelect={noop}
        filter="unconfirmed"
        onFilterChange={noop}
      />,
    )
    expect(screen.getByTestId('proceeding-row-V-1/2026-701')).toBeTruthy()
    expect(screen.getByTestId('proceeding-row-V-3/2026-701')).toBeTruthy()
    expect(screen.queryByTestId('proceeding-row-V-2/2026-701')).toBeNull()
  })

  it('shows the empty message when filter=unconfirmed and none match', () => {
    const allConfirmed = overview([proc({ cislo_rizeni: 'V-1/2026-701', unconfirmed_count: 0 })])
    render(
      <ProceedingsList
        data={allConfirmed}
        loading={false}
        error={null}
        refetch={noop}
        selectedCislo={null}
        onSelect={noop}
        filter="unconfirmed"
        onFilterChange={noop}
      />,
    )
    expect(screen.getByTestId('no-unconfirmed')).toBeTruthy()
    expect(screen.queryByTestId('proceeding-row-V-1/2026-701')).toBeNull()
  })
})
