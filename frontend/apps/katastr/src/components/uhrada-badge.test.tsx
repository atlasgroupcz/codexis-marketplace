// @vitest-environment jsdom

import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { UhradaBadge } from './uhrada-badge'

describe('UhradaBadge', () => {
  // The wire carries a Czech-only `stav_uhrady_label` built in Rust, so reading
  // it printed "Uhrazeno" to an English or Slovak reader. The code is the only
  // language-free part of the payload, so the badge translates that instead.
  it('names the payment state in the reader language, not the wire language', () => {
    render(<UhradaBadge stavUhrady="U" />)
    expect(screen.getByText('Paid')).toBeTruthy()
  })

  it('shows an unknown ČÚZK code rather than blanking', () => {
    render(<UhradaBadge stavUhrady="X" />)
    expect(screen.getByText('X')).toBeTruthy()
  })

  it('reads as absent when ČÚZK reports no payment state', () => {
    render(<UhradaBadge stavUhrady={null} />)
    expect(screen.getByText('---')).toBeTruthy()
  })
})
