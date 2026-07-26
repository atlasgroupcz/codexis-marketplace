import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { DetailResponse } from '@/lib/schemas'
import { DocumentsHome } from '@/components/documents-home'
import { renderApp } from '@/test/render-app'
import { detailFixture, detailWithPartsFixture, overviewFixture } from '@/test/fixtures'

const detailByUuid: Record<string, DetailResponse> = {
  '2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d': detailFixture,
  '6a6de088-1909-4d80-a764-d2fcb8ec4eb2': detailWithPartsFixture,
}

function emptyDetail(uuid: string): DetailResponse {
  return {
    mode: 'detail',
    generated_at: '2026-02-25T09:00:00Z',
    document: {
      uuid,
      codexisId: 'cdx://doc/UNKNOWN',
      name: 'Zákon o DPH (235/2004 Sb.)',
      added_on: '2026-02-25T07:40:00Z',
      tracking_type: 'related_documents_changes',
      parts: [],
      changes: [],
      total_changes: 0,
      unconfirmed_changes: 0,
      groups: [],
      user_notes: [],
    },
    groups: [],
  }
}

/** Returns overview for the base endpoint and the matching detail when ?uuid= is present. */
function stubRoutedFetch() {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockImplementation((input: unknown) => {
      const url = String(input)
      const match = /uuid=([^&]+)/.exec(url)
      if (match) {
        const uuid = decodeURIComponent(match[1] ?? '')
        const detail = detailByUuid[uuid] ?? emptyDetail(uuid)
        return Promise.resolve({ ok: true, json: () => Promise.resolve(detail) })
      }
      return Promise.resolve({ ok: true, json: () => Promise.resolve(overviewFixture) })
    }),
  )
}

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('DocumentsHome', () => {
  it('renders the branded header', async () => {
    stubRoutedFetch()
    renderApp(<DocumentsHome section="documents" onChangeSection={vi.fn()} onSelectDocument={vi.fn()} onOpenSettings={vi.fn()} />)

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Followed documents' })).toBeInTheDocument()
    })
  })

  it('builds a cross-document changes feed', async () => {
    stubRoutedFetch()
    renderApp(<DocumentsHome section="documents" onChangeSection={vi.fn()} onSelectDocument={vi.fn()} onOpenSettings={vi.fn()} />)

    await waitFor(() => {
      expect(
        screen.getByText('Novela zákoníku práce v oblasti pracovní doby a dovolené.'),
      ).toBeInTheDocument()
    })
  })

  it('lists documents and selects one from the Documents tab', async () => {
    stubRoutedFetch()
    const onSelectDocument = vi.fn()
    const user = userEvent.setup()
    renderApp(<DocumentsHome section="documents" onChangeSection={vi.fn()} onSelectDocument={onSelectDocument} onOpenSettings={vi.fn()} />, {
      searchParams: { tab: 'documents' },
    })

    await waitFor(() => {
      expect(screen.getByText('Zákoník práce (262/2006 Sb.)')).toBeInTheDocument()
    })
    expect(screen.getByText('Občanský zákoník (89/2012 Sb.)')).toBeInTheDocument()

    await user.click(screen.getByText('Zákoník práce (262/2006 Sb.)'))
    expect(onSelectDocument).toHaveBeenCalledWith('2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d')
  })

  it('shows an error when the overview request fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({ ok: false, status: 500, statusText: 'Internal Server Error' }),
    )
    renderApp(<DocumentsHome section="documents" onChangeSection={vi.fn()} onSelectDocument={vi.fn()} onOpenSettings={vi.fn()} />)

    await waitFor(() => {
      expect(screen.getByText(/HTTP 500/)).toBeInTheDocument()
    })
  })
})
