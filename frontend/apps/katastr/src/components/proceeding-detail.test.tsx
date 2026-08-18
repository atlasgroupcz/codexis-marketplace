// @vitest-environment jsdom

import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { DetailResponse } from '@/lib/schemas'
import { detailFixture, proceedingFixture } from '@/test/fixtures'
import { ProceedingDetail } from './proceeding-detail'

// vi.hoisted so these exist before the hoisted vi.mock factory references them.
const { refetch, state } = vi.hoisted(() => ({
  refetch: vi.fn(),
  // Mutable, so a test can put the component back before its request answered.
  state: { data: null as DetailResponse | null },
}))

vi.mock('@/hooks/use-detail', () => ({
  useDetail: () => ({
    data: state.data,
    loading: state.data === null,
    error: null,
    refetch,
  }),
}))
vi.mock('@/lib/api', () => ({
  confirmChange: vi.fn(() => Promise.resolve({ ok: true })),
  setLabel: vi.fn(),
  removeProceeding: vi.fn(),
}))

const summary = proceedingFixture({
  stav: 'Plomba',
  stav_uhrady: 'N',
  datum_prijeti: '2026-06-01',
  last_check_at: '2026-06-12',
  operace_count: 1,
  changes_count: 1,
  unconfirmed_count: 1,
})

const detail = detailFixture({
  ...summary,
  provedene_operace: [{ nazev: 'Zaplombování', datumProvedeni: '2026-06-01' }],
  changes: [
    {
      detected_on: '2026-06-12',
      old_stav: 'x',
      new_stav: 'Plomba',
      new_operations: [],
      stav_uhrady_changed: false,
      confirmed_on: null,
    },
  ],
})

beforeEach(() => {
  refetch.mockClear()
  state.data = { mode: 'detail', generated_at: '', proceeding: detail }
})

describe('ProceedingDetail - confirm refreshes overview', () => {
  it('calls onChanged after confirming so the sidebar unread dot clears', async () => {
    const onChanged = vi.fn()
    render(<ProceedingDetail cisloRizeni="V-1/2026-701" onBack={() => {}} onChanged={onChanged} />)
    // Radix TabsTrigger switches the active tab on mousedown (not click), so a
    // plain fireEvent.click never mounts the "zmeny" TabsContent in jsdom.
    fireEvent.mouseDown(screen.getByTestId('tab-zmeny'))
    fireEvent.click(screen.getByTestId('mark-all-read'))
    await waitFor(() => expect(onChanged).toHaveBeenCalled())
  })

  it('keeps the bulk action off the timeline tab so switching tabs shifts nothing', () => {
    render(<ProceedingDetail cisloRizeni="V-1/2026-701" onBack={() => {}} />)
    expect(screen.queryByTestId('mark-all-read')).toBeNull()
    fireEvent.mouseDown(screen.getByTestId('tab-zmeny'))
    expect(screen.getByTestId('mark-all-read')).toBeTruthy()
  })
})

describe('ProceedingDetail - the header does not wait for the request', () => {
  // Runs after the outer beforeEach, so it puts the request back in flight.
  beforeEach(() => {
    state.data = null
  })

  it('paints the header from the list row the sidebar already holds', () => {
    render(<ProceedingDetail cisloRizeni="V-1/2026-701" summary={summary} onBack={() => {}} />)
    // Selecting a proceeding used to blank the pane for a whole VM round-trip,
    // although the list had every one of these fields already.
    expect(screen.getByText('V-1/2026-701')).toBeTruthy()
    expect(screen.getByText('Unpaid')).toBeTruthy()
    expect(screen.getByText('Flagged')).toBeTruthy() // the badge for stav "Plomba"
  })

  it('still waits for the request before drawing the operations', () => {
    render(<ProceedingDetail cisloRizeni="V-1/2026-701" summary={summary} onBack={() => {}} />)
    // The row carries no operations and no change log - only the detail does.
    expect(screen.queryByTestId('timeline-node-op:0')).toBeNull()
  })

  it('falls back to the skeleton when there is no row to paint from', () => {
    render(<ProceedingDetail cisloRizeni="V-1/2026-701" onBack={() => {}} />)
    expect(screen.queryByText('V-1/2026-701')).toBeNull()
  })
})
