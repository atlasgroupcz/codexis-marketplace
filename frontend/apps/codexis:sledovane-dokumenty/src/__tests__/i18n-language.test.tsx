import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { DocumentList } from '@/components/document-list'
import { renderApp } from '@/test/render-app'
import { overviewFixture } from '@/test/fixtures'

beforeEach(() => {
  vi.restoreAllMocks()
  window.localStorage.clear()
})

describe('i18n language resolution', () => {
  it('renders English copy when URL has lang=en', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(overviewFixture),
        text: () => Promise.resolve('{}'),
      }),
    )

    renderApp(<DocumentList onSelectDocument={() => {}} />, {
      searchParams: { lang: 'en' },
    })

    await waitFor(() => {
      expect(
        screen.getByRole('heading', { name: 'Followed documents' }),
      ).toBeInTheDocument()
    })
  })

  it('uses localStorage language when URL lang is absent', async () => {
    window.localStorage.setItem('i18nextLng', 'sk')
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(overviewFixture),
        text: () => Promise.resolve('{}'),
      }),
    )

    renderApp(<DocumentList onSelectDocument={() => {}} />)

    await waitFor(() => {
      expect(screen.getByText('Všetko')).toBeInTheDocument()
    })
  })

  it('URL lang overrides localStorage language', async () => {
    window.localStorage.setItem('i18nextLng', 'cs')
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(overviewFixture),
        text: () => Promise.resolve('{}'),
      }),
    )

    renderApp(<DocumentList onSelectDocument={() => {}} />, {
      searchParams: { lang: 'en' },
    })

    await waitFor(() => {
      expect(
        screen.getByRole('heading', { name: 'Followed documents' }),
      ).toBeInTheDocument()
    })
  })
})
