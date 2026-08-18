// @vitest-environment jsdom

import { render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ErrorBoundary } from './error-boundary'

function Boom(): never {
  throw new Error('rozbito')
}

describe('ErrorBoundary', () => {
  // React logs the caught error itself; silence it so a passing run stays clean.
  beforeEach(() => vi.spyOn(console, 'error').mockImplementation(() => {}))
  afterEach(() => vi.restoreAllMocks())

  it('renders its children when nothing throws', () => {
    render(
      <ErrorBoundary>
        <p>obsah</p>
      </ErrorBoundary>,
    )
    expect(screen.getByText('obsah')).toBeTruthy()
  })

  it('shows the message instead of losing the page', () => {
    render(
      <ErrorBoundary>
        <Boom />
      </ErrorBoundary>,
    )
    // Without this the iframe would be blank, with nothing to read or click.
    expect(screen.getByText(/rozbito/)).toBeTruthy()
  })

  it('names the failing area when given a title', () => {
    render(
      <ErrorBoundary title="Katastr">
        <Boom />
      </ErrorBoundary>,
    )
    expect(screen.getByText(/Katastr: rozbito/)).toBeTruthy()
  })

  it('offers a way out', () => {
    render(
      <ErrorBoundary retryLabel="Načíst znovu">
        <Boom />
      </ErrorBoundary>,
    )
    expect(screen.getByRole('button', { name: 'Načíst znovu' })).toBeTruthy()
  })
})
