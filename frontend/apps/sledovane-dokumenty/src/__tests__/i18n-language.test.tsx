import { screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { DocumentsHome } from '@/components/documents-home'
import { overviewFixture } from '@/test/fixtures'
import { renderApp } from '@/test/render-app'

beforeEach(() => {
  vi.restoreAllMocks()
  window.localStorage.clear()
})

function stubOverview() {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(overviewFixture),
      text: () => Promise.resolve('{}'),
    }),
  )
}

describe('i18n language resolution', () => {
  it('renders English copy when URL has lang=en', async () => {
    stubOverview()
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={() => {}}
        onSelectDocument={() => {}}
        onOpenSettings={() => {}}
      />,
      { searchParams: { lang: 'en' } },
    )

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Followed documents' })).toBeInTheDocument()
    })
  })

  it('uses localStorage language when URL lang is absent', async () => {
    window.localStorage.setItem('i18nextLng', 'sk')
    stubOverview()
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={() => {}}
        onSelectDocument={() => {}}
        onOpenSettings={() => {}}
      />,
    )

    await waitFor(() => {
      expect(screen.getByText('Strážime legislatívu, na ktorej vám záleží')).toBeInTheDocument()
    })
  })

  it('URL lang overrides localStorage language', async () => {
    window.localStorage.setItem('i18nextLng', 'cs')
    stubOverview()
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={() => {}}
        onSelectDocument={() => {}}
        onOpenSettings={() => {}}
      />,
      { searchParams: { lang: 'en' } },
    )

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Followed documents' })).toBeInTheDocument()
    })
  })
})
