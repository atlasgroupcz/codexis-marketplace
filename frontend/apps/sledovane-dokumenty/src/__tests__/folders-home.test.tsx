import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FoldersHome } from '@/components/folders-home'
import type { FoldersResponse, OverviewResponse } from '@/lib/schemas'
import { overviewFixture } from '@/test/fixtures'
import { renderApp } from '@/test/render-app'

const foldersFixture: FoldersResponse = {
  mode: 'folders',
  generated_at: '2026-06-01T12:00:00Z',
  folders: [
    {
      uuid: 'u1',
      root: '/home/u/my-contracts',
      name: 'Smlouvy',
      added_on: '2026-06-01T12:00:00Z',
      documents: 3,
      pending_harvest: 1,
      legislation: 2,
      unconfirmed_changes: 1,
      last_check_at: null,
    },
  ],
  settings: { email: true, inApp: true, recipients: [] },
}

const emptyFolders: FoldersResponse = {
  mode: 'folders',
  generated_at: '2026-06-01T12:00:00Z',
  folders: [],
  settings: { email: true, inApp: true, recipients: [] },
}

/** Folders for this screen, overview for the act badge the rail fetches itself. */
function stubFolders(payload: FoldersResponse, overview: OverviewResponse = overviewFixture) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockImplementation((input: unknown) =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve(String(input).includes('view=folders') ? payload : overview),
      }),
    ),
  )
}

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('FoldersHome', () => {
  it('renders folder cards from the overview', async () => {
    stubFolders(foldersFixture)
    renderApp(
      <FoldersHome
        section="folders"
        onChangeSection={vi.fn()}
        onSelectFolder={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
      {
        searchParams: { lang: 'en' },
      },
    )

    await waitFor(() => {
      expect(screen.getByText('Smlouvy')).toBeInTheDocument()
    })
    expect(screen.getByText('/home/u/my-contracts')).toBeInTheDocument()
    expect(screen.getByText('3 documents')).toBeInTheDocument()
    expect(screen.getByText('2 acts')).toBeInTheDocument()
  })

  it('shows the empty landing when no folders are watched', async () => {
    stubFolders(emptyFolders)
    renderApp(
      <FoldersHome
        section="folders"
        onChangeSection={vi.fn()}
        onSelectFolder={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
      {
        searchParams: { lang: 'en' },
      },
    )

    await waitFor(() => {
      expect(screen.getByText('Watch the legislation in your documents')).toBeInTheDocument()
    })
  })

  it('keeps the app identity in the header across sections', async () => {
    stubFolders(foldersFixture)
    renderApp(
      <FoldersHome
        section="folders"
        onChangeSection={vi.fn()}
        onSelectFolder={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
      {
        searchParams: { lang: 'en' },
      },
    )

    // the folder mode is named by the section tab, not by retitling the app
    expect(await screen.findByRole('heading', { name: 'Followed documents' })).toBeInTheDocument()
  })

  it('switches back to the act watchdog from the section navigation', async () => {
    stubFolders(foldersFixture)
    const onChangeSection = vi.fn()
    const user = userEvent.setup()
    renderApp(
      <FoldersHome
        section="folders"
        onChangeSection={onChangeSection}
        onSelectFolder={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
      {
        searchParams: { lang: 'en' },
      },
    )

    const folders = await screen.findByRole('button', { name: /Folders/ })
    expect(folders).toHaveAttribute('aria-current', 'page')

    await user.click(screen.getByRole('button', { name: /Acts/ }))
    expect(onChangeSection).toHaveBeenCalledWith('documents')
  })

  it('badges the act watchdog with the changes waiting there', async () => {
    stubFolders(foldersFixture)
    renderApp(
      <FoldersHome
        section="folders"
        onChangeSection={vi.fn()}
        onSelectFolder={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
      {
        searchParams: { lang: 'en' },
      },
    )

    // the overview fixture carries two unconfirmed act changes
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Acts/ })).toHaveTextContent('2')
    })
  })

  it('offers adding a folder from the header', async () => {
    stubFolders(foldersFixture)
    renderApp(
      <FoldersHome
        section="folders"
        onChangeSection={vi.fn()}
        onSelectFolder={vi.fn()}
        onOpenSettings={vi.fn()}
      />,
      {
        searchParams: { lang: 'en' },
      },
    )

    expect(await screen.findByRole('button', { name: /Add folder/ })).toBeInTheDocument()
  })

  it('selects a folder when its card is clicked', async () => {
    stubFolders(foldersFixture)
    const onSelectFolder = vi.fn()
    const user = userEvent.setup()
    renderApp(
      <FoldersHome
        section="folders"
        onChangeSection={vi.fn()}
        onSelectFolder={onSelectFolder}
        onOpenSettings={vi.fn()}
      />,
      { searchParams: { lang: 'en' } },
    )

    await waitFor(() => expect(screen.getByText('Smlouvy')).toBeInTheDocument())
    await user.click(screen.getByText('Smlouvy'))
    expect(onSelectFolder).toHaveBeenCalledWith('/home/u/my-contracts')
  })
})
