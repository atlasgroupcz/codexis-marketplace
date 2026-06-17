import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { FoldersResponse } from '@/lib/schemas'
import { FoldersHome } from '@/components/folders-home'
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

function stubFolders(payload: FoldersResponse) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve(payload) }),
  )
}

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('FoldersHome', () => {
  it('renders folder cards from the overview', async () => {
    stubFolders(foldersFixture)
    renderApp(<FoldersHome section="folders" onChangeSection={vi.fn()} onSelectFolder={vi.fn()} />, {
      searchParams: { lang: 'en' },
    })

    await waitFor(() => {
      expect(screen.getByText('Smlouvy')).toBeInTheDocument()
    })
    expect(screen.getByText('/home/u/my-contracts')).toBeInTheDocument()
    expect(screen.getByText('3 documents')).toBeInTheDocument()
    expect(screen.getByText('2 acts')).toBeInTheDocument()
  })

  it('shows the empty landing when no folders are watched', async () => {
    stubFolders(emptyFolders)
    renderApp(<FoldersHome section="folders" onChangeSection={vi.fn()} onSelectFolder={vi.fn()} />, {
      searchParams: { lang: 'en' },
    })

    await waitFor(() => {
      expect(
        screen.getByText('Watch the legislation in your documents'),
      ).toBeInTheDocument()
    })
  })

  it('selects a folder when its card is clicked', async () => {
    stubFolders(foldersFixture)
    const onSelectFolder = vi.fn()
    const user = userEvent.setup()
    renderApp(
      <FoldersHome section="folders" onChangeSection={vi.fn()} onSelectFolder={onSelectFolder} />,
      { searchParams: { lang: 'en' } },
    )

    await waitFor(() => expect(screen.getByText('Smlouvy')).toBeInTheDocument())
    await user.click(screen.getByText('Smlouvy'))
    expect(onSelectFolder).toHaveBeenCalledWith('/home/u/my-contracts')
  })
})
