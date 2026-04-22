import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { DocumentDetail } from '@/components/document-detail'
import { renderApp } from '@/test/render-app'
import { detailFixture, detailWithPartsFixture } from '@/test/fixtures'

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('DocumentDetail', () => {
  it('renders document name and codexisId', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(detailFixture),
      }),
    )

    const onBack = vi.fn()
    renderApp(
      <DocumentDetail uuid="2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d" onBack={onBack} />,
    )

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Zákoník práce (262/2006 Sb.)' })).toBeInTheDocument()
    })
    expect(screen.getByText('cdx://doc/CR13986')).toBeInTheDocument()
  })

  it('renders changes', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(detailFixture),
      }),
    )

    const onBack = vi.fn()
    renderApp(
      <DocumentDetail uuid="2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d" onBack={onBack} />,
    )

    await waitFor(() => {
      expect(
        screen.getByText('Novela zákoníku práce v oblasti pracovní doby a dovolené.'),
      ).toBeInTheDocument()
    })
  })

  it('renders tracked parts as badges', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(detailWithPartsFixture),
      }),
    )

    const onBack = vi.fn()
    renderApp(
      <DocumentDetail uuid="6a6de088-1909-4d80-a764-d2fcb8ec4eb2" onBack={onBack} />,
    )

    await waitFor(() => {
      expect(screen.getByText('\u00a7 89')).toBeInTheDocument()
    })
    expect(screen.getByText('\u00a7 2991')).toBeInTheDocument()
  })

  it('calls onBack when clicking back button', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(detailFixture),
      }),
    )

    const user = userEvent.setup()
    const onBack = vi.fn()
    renderApp(
      <DocumentDetail uuid="2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d" onBack={onBack} />,
    )

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Zákoník práce (262/2006 Sb.)' })).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: /back/i }))
    expect(onBack).toHaveBeenCalled()
  })

  it('shows error message on fetch failure', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 404,
        statusText: 'Not Found',
      }),
    )

    const onBack = vi.fn()
    renderApp(
      <DocumentDetail uuid="nonexistent" onBack={onBack} />,
    )

    await waitFor(() => {
      expect(screen.getByText(/HTTP 404/)).toBeInTheDocument()
    })
  })
})
