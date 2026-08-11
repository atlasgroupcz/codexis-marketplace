import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { ErrorMessage } from './error-message'

describe('ErrorMessage', () => {
  it('renders the message', () => {
    render(<ErrorMessage message="Selhalo načtení" />)
    expect(screen.getByText('Selhalo načtení')).toBeInTheDocument()
  })

  it('renders retry button and fires onRetry', () => {
    const onRetry = vi.fn()
    render(<ErrorMessage message="x" onRetry={onRetry} retryLabel="Zkusit znovu" />)
    fireEvent.click(screen.getByRole('button', { name: 'Zkusit znovu' }))
    expect(onRetry).toHaveBeenCalledOnce()
  })

  it('omits retry button when no onRetry', () => {
    render(<ErrorMessage message="x" />)
    expect(screen.queryByRole('button')).toBeNull()
  })
})
