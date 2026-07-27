import { describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { Landmark } from 'lucide-react'
import { AppHeader, HeaderGear, HeaderHelp } from './app-header'

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

  it('opens the manual page of the app in the current language', () => {
    render(<AppHeader actions={<HeaderHelp app="katastr" lang="cs" title="Nápověda" />} />)
    const link = screen.getByRole('link', { name: 'Nápověda' })
    expect(link).toHaveAttribute(
      'href',
      'https://docs.codexis.ai/codexis-ai-2-0/manual/aplikace/katastr-sledovana-rizeni/',
    )
    expect(link).toHaveAttribute('target', '_blank')
  })
})
