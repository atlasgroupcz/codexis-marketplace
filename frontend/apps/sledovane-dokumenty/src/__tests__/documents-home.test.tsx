import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { DocumentsHome } from '@/components/documents-home'
import type { DetailResponse, WatchesResponse } from '@/lib/schemas'
import { detailFixture, detailWithPartsFixture, overviewFixture } from '@/test/fixtures'
import { renderApp } from '@/test/render-app'

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

/** The rail fetches the project badge itself; four unconfirmed across two projects. */
const watchesFixture: WatchesResponse = {
  mode: 'projects',
  generated_at: '2026-06-01T12:00:00Z',
  watches: [
    {
      uuid: 'w1',
      projectId: 'p-contracts',
      path: '.',
      name: 'Smlouvy',
      documents: 3,
      pending_harvest: 0,
      legislation: 2,
      unconfirmed_changes: 3,
    },
    {
      uuid: 'w2',
      projectId: 'p-policies',
      path: 'smernice',
      name: 'Směrnice',
      documents: 1,
      pending_harvest: 0,
      legislation: 1,
      unconfirmed_changes: 1,
    },
  ],
  settings: { email: true, inApp: true, recipients: [] },
}

/** Detail when ?uuid= is present, projects for the rail badge, overview otherwise. */
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
      if (url.includes('view=projects')) {
        return Promise.resolve({ ok: true, json: () => Promise.resolve(watchesFixture) })
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
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={vi.fn()}
        onSelectDocument={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
    )

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Followed documents' })).toBeInTheDocument()
    })
  })

  it('builds a cross-document changes feed', async () => {
    stubRoutedFetch()
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={vi.fn()}
        onSelectDocument={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
    )

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
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={vi.fn()}
        onSelectDocument={onSelectDocument}
        onOpenSettings={vi.fn()}
      />,
      {
        searchParams: { tab: 'documents' },
      },
    )

    await waitFor(() => {
      expect(screen.getByText('Zákoník práce (262/2006 Sb.)')).toBeInTheDocument()
    })
    expect(screen.getByText('Občanský zákoník (89/2012 Sb.)')).toBeInTheDocument()

    await user.click(screen.getByText('Zákoník práce (262/2006 Sb.)'))
    expect(onSelectDocument).toHaveBeenCalledWith('2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d')
  })

  it('opens a pre-filled chat draft from the header once documents are tracked', async () => {
    stubRoutedFetch()
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={vi.fn()}
        onSelectDocument={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
    )

    const link = await screen.findByTestId('add-monitoring')
    expect(link).toHaveAttribute(
      'href',
      `/?draft=${encodeURIComponent('Track changes in the Labour Code')}`,
    )
    expect(link).toHaveAttribute('target', '_top')
  })

  it('hides the header draft link while nothing is tracked', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ ...overviewFixture, tracked_documents: [] }),
      }),
    )
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={vi.fn()}
        onSelectDocument={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
    )

    // the landing screen carries the example prompt instead
    await waitFor(() => {
      expect(screen.getByTestId('empty-chat-example')).toBeInTheDocument()
    })
    expect(screen.queryByTestId('add-monitoring')).not.toBeInTheDocument()
  })

  it('switches to the project watchdog from the section navigation', async () => {
    stubRoutedFetch()
    const onChangeSection = vi.fn()
    const user = userEvent.setup()
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={onChangeSection}
        onSelectDocument={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
    )

    const acts = await screen.findByRole('button', { name: /Acts/ })
    expect(acts).toHaveAttribute('aria-current', 'page')

    await user.click(screen.getByRole('button', { name: /Projects/ }))
    expect(onChangeSection).toHaveBeenCalledWith('projects')
  })

  it('badges the project watchdog with the changes waiting there', async () => {
    stubRoutedFetch()
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={vi.fn()}
        onSelectDocument={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
    )

    // three plus one unconfirmed across the two projects it fetched for the badge
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Projects/ })).toHaveTextContent('4')
    })
  })

  it('badges its own watchdog from its own data', async () => {
    stubRoutedFetch()
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={vi.fn()}
        onSelectDocument={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
    )

    // the overview fixture carries two unconfirmed act changes
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Acts/ })).toHaveTextContent('2')
    })
  })

  it('shows an error when the overview request fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({ ok: false, status: 500, statusText: 'Internal Server Error' }),
    )
    renderApp(
      <DocumentsHome
        section="documents"
        onChangeSection={vi.fn()}
        onSelectDocument={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
    )

    await waitFor(() => {
      expect(screen.getByText(/HTTP 500/)).toBeInTheDocument()
    })
  })
})
