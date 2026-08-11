import { fireEvent, render, screen } from '@testing-library/react'
import { KeyRound } from 'lucide-react'
import { describe, expect, it, vi } from 'vitest'
import { ConfigGate } from './config-gate'

describe('ConfigGate', () => {
  it('renders title/text and fires the action', () => {
    const onClick = vi.fn()
    render(
      <ConfigGate
        icon={KeyRound}
        title="Připojte katastr"
        text="Zadejte API klíč ČÚZK."
        action={{ label: 'Zadat klíč', onClick }}
      />,
    )
    expect(screen.getByText('Připojte katastr')).toBeInTheDocument()
    expect(screen.getByText('Zadejte API klíč ČÚZK.')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Zadat klíč' }))
    expect(onClick).toHaveBeenCalledOnce()
  })
})
