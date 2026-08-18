import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ConversationHint } from './conversation-hint'

describe('ConversationHint', () => {
  it('renders lead and prompt, links to /?draft=', () => {
    render(<ConversationHint lead="Zkuste v chatu:" prompt="Sleduj řízení V-1/2026" />)
    expect(screen.getByText('Zkuste v chatu:')).toBeInTheDocument()
    const link = screen.getByTestId('empty-chat-example')
    expect(link.getAttribute('href')).toBe(
      `/?draft=${encodeURIComponent('Sleduj řízení V-1/2026')}`,
    )
  })
})
