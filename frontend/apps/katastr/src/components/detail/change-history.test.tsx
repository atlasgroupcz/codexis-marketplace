// @vitest-environment jsdom

import { fireEvent, render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ProceedingChange } from '@/lib/schemas'
import { ChangeHistory } from './change-history'

const onConfirm = vi.fn()

const change = (over: Partial<ProceedingChange>): ProceedingChange =>
  ({
    detected_on: '2026-05-18T09:12:00',
    old_stav: 'Provedení vkladu',
    new_stav: 'Ukončení řízení',
    new_operations: [],
    stav_uhrady_changed: false,
    confirmed_on: null,
    ...over,
  }) as ProceedingChange

const render3 = (changes: Array<ProceedingChange>) =>
  render(<ChangeHistory changes={changes} onConfirm={onConfirm} confirming={false} />)

describe('ChangeHistory', () => {
  beforeEach(() => onConfirm.mockClear())

  it('explains itself instead of rendering an empty log', () => {
    render3([])
    expect(screen.getByTestId('no-changes')).toBeTruthy()
  })

  it('names its columns and orders rows newest first', () => {
    render3([
      change({ detected_on: '2026-04-24T07:30:00', new_stav: 'Zaplombování' }),
      change({ detected_on: '2026-05-18T09:12:00', new_stav: 'Ukončení řízení' }),
    ])
    expect(screen.getByText('Detected')).toBeTruthy()
    expect(screen.getByText('Status')).toBeTruthy()
    const states = screen.getAllByText(/Zaplombování|Ukončení řízení/)
    expect(states[0]?.textContent).toBe('Ukončení řízení')
    expect(states[1]?.textContent).toBe('Zaplombování')
  })

  it('heads the confirm column with the bulk action', () => {
    render3([change({})])
    const header = screen.getByTestId('mark-all-read').closest('th')
    expect(header).toBeTruthy()
    expect(screen.getByTestId('mark-read-0').closest('td')).toBeTruthy()
  })

  it('hides the bulk action once everything is read', () => {
    render3([change({ confirmed_on: '2026-05-19T08:00:00' })])
    expect(screen.queryByTestId('mark-all-read')).toBeNull()
  })

  it('confirms every unread change when the bulk action is used', () => {
    render3([change({}), change({ detected_on: '2026-04-24T07:30:00' })])
    fireEvent.click(screen.getByTestId('mark-all-read'))
    expect(onConfirm).toHaveBeenCalledWith()
  })

  it('prints only the new state, never the one it came from', () => {
    render3([
      change({
        new_operations: [
          { nazev: 'Provedení vkladu', datumProvedeni: '2026-05-15T13:41:00' },
          { nazev: 'Vyrozumění o provedení vkladu', datumProvedeni: '2026-05-15T13:52:00' },
        ],
      }),
    ])
    expect(screen.getByText('Ukončení řízení')).toBeTruthy()
    // the previous state is the entry below it, and the operations are the Průběh tab
    expect(screen.queryByText(/Provedení vkladu/)).toBeNull()
    expect(screen.queryByText(/→/)).toBeNull()
  })

  it('names the new operations when the state itself did not move', () => {
    render3([
      change({
        old_stav: 'Probíhá zpracování',
        new_stav: 'Probíhá zpracování',
        new_operations: [{ nazev: 'Zaplombování', datumProvedeni: '2026-05-15T13:41:00' }],
      }),
    ])
    // otherwise the entry would have nothing to say at all
    expect(screen.getByText('Zaplombování')).toBeTruthy()
  })

  it('pads the hour so a column of times keeps its colons in line', () => {
    render3([
      change({ detected_on: '2026-05-18T08:22:00' }),
      change({ detected_on: '2026-04-24T07:30:00' }),
    ])
    expect(screen.getByText(/08:22/)).toBeTruthy()
  })

  it('confirms a single change by its index in the unsorted list', () => {
    render3([
      change({ detected_on: '2026-04-24T07:30:00' }),
      change({ detected_on: '2026-05-18T09:12:00' }),
    ])
    // The oldest renders last but is index 0 in `proc.changes`, which is the id
    // the API takes.
    fireEvent.click(screen.getByTestId('mark-read-0'))
    expect(onConfirm).toHaveBeenCalledWith(0)
  })

  it('offers no confirm control on an already-read change', () => {
    render3([change({ confirmed_on: '2026-05-19T08:00:00' })])
    expect(screen.queryByTestId('mark-read-0')).toBeNull()
  })

  it('carries a payment flip as the line when nothing else moved', () => {
    render3([
      change({
        old_stav: 'Probíhá',
        new_stav: 'Probíhá',
        stav_uhrady_changed: true,
        old_stav_uhrady: 'N',
        new_stav_uhrady: 'U',
      }),
    ])
    expect(screen.getByText('Paid')).toBeTruthy()
    // an unchanged stav is not restated as a transition to itself
    expect(screen.queryByText(/Probíhá/)).toBeNull()
  })

  it('appends a payment flip that rode along with a state change', () => {
    const { container } = render3([
      change({ stav_uhrady_changed: true, old_stav_uhrady: 'N', new_stav_uhrady: 'U' }),
    ])
    // both facts survive without the entry growing a line of its own
    expect(container.textContent).toContain('Ukončení řízení')
    expect(container.textContent).toContain('Paid')
  })

  it('appends a payment flip that rode along with new operations', () => {
    // The payment used to be printed only beside a state change, so a check that
    // found an operation and a payment at once showed the operation and dropped
    // the payment entirely - the CLI and the e-mail reported both.
    const { container } = render3([
      change({
        old_stav: 'Probíhá zpracování',
        new_stav: 'Probíhá zpracování',
        new_operations: [{ nazev: 'Zaplombování', datumProvedeni: '2026-05-15T13:41:00' }],
        stav_uhrady_changed: true,
        old_stav_uhrady: 'N',
        new_stav_uhrady: 'U',
      }),
    ])
    expect(container.textContent).toContain('Zaplombování')
    expect(container.textContent).toContain('Paid')
  })

  it('leaves no dangling separator when the payment is the whole entry', () => {
    const { container } = render3([
      change({
        old_stav: 'Probíhá',
        new_stav: 'Probíhá',
        stav_uhrady_changed: true,
        old_stav_uhrady: 'N',
        new_stav_uhrady: 'U',
      }),
    ])
    expect(container.querySelectorAll('tbody td')[1]?.textContent?.trim()).toBe('Paid')
  })

  it('lets neither the columns nor the rows be sized by what a row holds', () => {
    // Confirming removes buttons - the row's, and on the last one the header's -
    // and with the browser's defaults each removal re-measured the table and
    // moved everything under it.
    const { container } = render3([change({ confirmed_on: '2026-05-19T08:00:00' })])
    expect(container.querySelector('table')?.className).toContain('table-fixed')
    expect(container.querySelectorAll('col')).toHaveLength(3)
  })

  it('keeps a read row the same height by holding the button it no longer needs', () => {
    // The button is taller than the text beside it, so removing it shrank the
    // row. Its own box reserves the space, which beats guessing a row height.
    const { container } = render3([change({ confirmed_on: '2026-05-19T08:00:00' })])
    const button = container.querySelector('tbody button')
    expect(button?.className).toContain('invisible')
    expect(button?.getAttribute('aria-hidden')).toBe('true')
    expect(button?.getAttribute('tabindex')).toBe('-1')
    expect((button as HTMLButtonElement | null)?.disabled).toBe(true)
  })

  it('keeps the header the same height once everything is read', () => {
    // The header sits above the list, so its height moved every row below it.
    const { container } = render3([change({ confirmed_on: '2026-05-19T08:00:00' })])
    expect(container.querySelector('thead button')?.className).toContain('invisible')
  })

  it('marks a read row by colour, not by weight', () => {
    const { container } = render3([
      change({ confirmed_on: '2026-05-19T08:00:00', new_stav: 'Ukončení řízení' }),
    ])
    // Bold text is wider, so a weight change could re-wrap a long state name and
    // move the row's own height.
    const cell = container.querySelectorAll('tbody td')[1]
    expect(cell?.className).toContain('font-medium')
    expect(cell?.className).toContain('text-muted-foreground')
  })
})
