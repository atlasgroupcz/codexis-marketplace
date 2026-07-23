// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { ProceedingDetail } from './proceeding-detail'

// vi.hoisted so these exist before the hoisted vi.mock factories reference them.
const { refetch, detail } = vi.hoisted(() => ({
  refetch: vi.fn(),
  detail: {
    cislo_rizeni: 'V-1/2026-701', typ_rizeni: 'Vklad', label: '',
    stav: 'Plomba', stav_uhrady: 'N', stav_uhrady_label: 'Neuhrazeno',
    datum_prijeti: '2026-06-01', added_on: '2026-06-01', last_check_at: '2026-06-12',
    last_op_date: '2026-06-12', operace_count: 1, changes_count: 1, unconfirmed_count: 1,
    provedene_operace: [{ nazev: 'Zaplombování', datumProvedeni: '2026-06-01' }],
    changes: [{
      detected_on: '2026-06-12', old_stav: 'x', new_stav: 'Plomba',
      new_operations: [], stav_uhrady_changed: false, confirmed_on: null,
    }],
  },
}))

vi.mock('@/hooks/use-detail', () => ({
  useDetail: () => ({
    data: { mode: 'detail', generated_at: '', proceeding: detail },
    loading: false, error: null, refetch,
  }),
}))
vi.mock('@/lib/api', () => ({
  confirmChange: vi.fn(() => Promise.resolve({ ok: true })),
  setLabel: vi.fn(), removeProceeding: vi.fn(),
}))

describe('ProceedingDetail - confirm refreshes overview', () => {
  beforeEach(() => refetch.mockClear())

  it('calls onChanged after confirming so the sidebar unread dot clears', async () => {
    const onChanged = vi.fn()
    render(<ProceedingDetail cisloRizeni="V-1/2026-701" onBack={() => {}} onChanged={onChanged} />)
    // Radix TabsTrigger switches the active tab on mousedown (not click), so a
    // plain fireEvent.click never mounts the "zmeny" TabsContent in jsdom.
    fireEvent.mouseDown(screen.getByTestId('tab-zmeny'))
    fireEvent.click(screen.getByTestId('mark-all-read'))
    await waitFor(() => expect(onChanged).toHaveBeenCalled())
  })
})
