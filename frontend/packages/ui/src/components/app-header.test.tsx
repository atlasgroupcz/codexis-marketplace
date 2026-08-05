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

  it('shows the manual link next to the brand title', () => {
    render(
      <AppHeader
        brand={{ icon: Landmark, title: 'Katastr' }}
        docs={{ app: 'katastr', lang: 'cs', label: 'Jak to funguje?' }}
      />,
    )
    const link = screen.getByRole('link', { name: 'Jak to funguje?' })
    expect(link).toHaveAttribute(
      'href',
      'https://docs.codexis.ai/codexis-ai-2-0/manual/aplikace/katastr-sledovana-rizeni/',
    )
    expect(link).toHaveAttribute('target', '_blank')
  })

  it('shows the manual link on detail screens too', () => {
    render(
      <AppHeader
        back={{ onClick: () => {}, label: 'Zpět', title: 'Detail tématu' }}
        docs={{ app: 'sledovana-judikatura', lang: 'en', label: 'How does it work?' }}
      />,
    )
    expect(screen.getByRole('link', { name: 'How does it work?' })).toHaveAttribute(
      'href',
      'https://docs.codexis.ai/en/codexis-ai-2-0/manual/aplikace/sledovana-judikatura/',
    )
  })

  it('drops the manual link when there is no title to hang it on', () => {
    render(<AppHeader docs={{ app: 'katastr', lang: 'cs', label: 'Jak to funguje?' }} />)
    expect(screen.queryByRole('link')).not.toBeInTheDocument()
  })
})
