import { describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { Landmark } from 'lucide-react'
import { AppHeader, HeaderGear } from './app-header'

describe('AppHeader', () => {
  it('renders brand title + subtitle + actions', () => {
    render(
      <AppHeader
        brand={{ icon: Landmark, title: 'Katastr', subtitle: 'Sledovaná řízení' }}
        actions={<HeaderGear onClick={() => {}} title="Nastavení" />}
      />,
    )
    expect(screen.getByText('Katastr')).toBeInTheDocument()
    expect(screen.getByText('Sledovaná řízení')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Nastavení' })).toBeInTheDocument()
  })

  it('renders back button and fires onClick in back mode', () => {
    const onBack = vi.fn()
    render(<AppHeader back={{ onClick: onBack, label: 'Zpět', title: 'Detail' }} />)
    fireEvent.click(screen.getByRole('button', { name: 'Zpět' }))
    expect(onBack).toHaveBeenCalledOnce()
  })
})
