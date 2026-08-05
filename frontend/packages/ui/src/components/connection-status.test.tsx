import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ConnectionStatus, connStateOf } from './connection-status'

const labels = { connected: 'Připojeno', notConnected: 'Nepřipojeno' }

describe('connStateOf', () => {
  it('reports an unanswered fetch as unknown, not as disconnected', () => {
    expect(connStateOf(null)).toBe('unknown')
    expect(connStateOf(undefined)).toBe('unknown')
  })

  it('maps a real answer to connected or disconnected', () => {
    expect(connStateOf(true)).toBe('connected')
    expect(connStateOf(false)).toBe('disconnected')
  })
})

describe('ConnectionStatus', () => {
  it('says nothing while the state is unknown', () => {
    render(<ConnectionStatus state="unknown" labels={labels} />)
    expect(screen.queryByText('Nepřipojeno')).toBeNull()
    expect(screen.queryByText('Připojeno')).toBeNull()
  })

  // The whole point: a panel must not open by asserting, in red, a state it has
  // not checked. The row still has to hold its height so nothing jumps.
  it('keeps the row height while unknown', () => {
    const { container } = render(<ConnectionStatus state="unknown" labels={labels} />)
    expect(container.querySelector('.h-4')).not.toBeNull()
  })

  it('reports each answered state with its own wording', () => {
    const { rerender } = render(<ConnectionStatus state="connected" labels={labels} />)
    expect(screen.getByText('Připojeno')).toBeTruthy()

    rerender(<ConnectionStatus state="disconnected" labels={labels} />)
    expect(screen.getByText('Nepřipojeno')).toBeTruthy()
  })

  it('marks a disconnected state as destructive whatever the tone', () => {
    const { container } = render(
      <ConnectionStatus state="disconnected" labels={labels} tone="success" />,
    )
    expect(container.querySelector('.text-destructive')).not.toBeNull()
  })

  // Most apps' primary IS the brand colour; datify's is Datify blue, so green
  // has to be asked for by name there.
  it('takes the connected colour from the tone', () => {
    const { container: primary } = render(<ConnectionStatus state="connected" labels={labels} />)
    expect(primary.querySelector('.text-primary')).not.toBeNull()

    const { container: success } = render(
      <ConnectionStatus state="connected" labels={labels} tone="success" />,
    )
    expect(success.querySelector('.text-primary')).toBeNull()
  })
})
