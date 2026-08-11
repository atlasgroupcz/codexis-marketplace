// @vitest-environment jsdom

import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { ListFilter } from './list-filter'

describe('ListFilter', () => {
  it('renders both segments and the unconfirmed count', () => {
    render(<ListFilter value="all" onChange={() => {}} unconfirmedCount={3} />)
    expect(screen.getByTestId('filter-all')).toBeTruthy()
    expect(screen.getByTestId('filter-unconfirmed')).toBeTruthy()
    expect(screen.getByTestId('filter-unconfirmed-count').textContent).toBe('3')
  })

  it('marks the active segment with aria-pressed', () => {
    render(<ListFilter value="unconfirmed" onChange={() => {}} unconfirmedCount={1} />)
    expect(screen.getByTestId('filter-unconfirmed').getAttribute('aria-pressed')).toBe('true')
    expect(screen.getByTestId('filter-all').getAttribute('aria-pressed')).toBe('false')
  })

  it('calls onChange with the segment value on click', () => {
    const onChange = vi.fn()
    render(<ListFilter value="all" onChange={onChange} unconfirmedCount={0} />)
    fireEvent.click(screen.getByTestId('filter-unconfirmed'))
    expect(onChange).toHaveBeenCalledWith('unconfirmed')
  })
})
