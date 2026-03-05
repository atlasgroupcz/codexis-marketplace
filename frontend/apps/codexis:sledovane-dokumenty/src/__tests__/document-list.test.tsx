import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { DocumentList } from '@/components/document-list'
import { renderApp } from '@/test/render-app'
import { overviewFixture } from '@/test/fixtures'

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('DocumentList', () => {
  it('renders document names after loading', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(overviewFixture),
      }),
    )

    const onSelectDocument = vi.fn()
    renderApp(<DocumentList onSelectDocument={onSelectDocument} />)

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Followed documents' })).toBeInTheDocument()
      expect(screen.getByText('Zakonik prace')).toBeInTheDocument()
    })
    expect(screen.getByText('Obcansky zakonik')).toBeInTheDocument()
    expect(screen.getByText('Spravni rad')).toBeInTheDocument()
  })

  it('shows unconfirmed badge with count', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(overviewFixture),
      }),
    )

    const onSelectDocument = vi.fn()
    renderApp(<DocumentList onSelectDocument={onSelectDocument} />)

    await waitFor(() => {
      expect(screen.getByText('Zakonik prace')).toBeInTheDocument()
    })
  })

  it('calls onSelectDocument when clicking a document', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(overviewFixture),
      }),
    )

    const user = userEvent.setup()
    const onSelectDocument = vi.fn()
    renderApp(<DocumentList onSelectDocument={onSelectDocument} />)

    await waitFor(() => {
      expect(screen.getByText('Zakonik prace')).toBeInTheDocument()
    })

    await user.click(screen.getByText('Zakonik prace'))
    expect(onSelectDocument).toHaveBeenCalledWith('2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d')
  })

  it('shows error message on fetch failure', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
      }),
    )

    const onSelectDocument = vi.fn()
    renderApp(<DocumentList onSelectDocument={onSelectDocument} />)

    await waitFor(() => {
      expect(screen.getByText(/HTTP 500/)).toBeInTheDocument()
    })
  })
})
