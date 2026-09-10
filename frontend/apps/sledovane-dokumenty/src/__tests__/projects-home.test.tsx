import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ProjectsHome } from '@/components/projects-home'
import type {
  AvailableProjectsResponse,
  BrowseResponse,
  OverviewResponse,
  WatchesResponse,
} from '@/lib/schemas'
import { overviewFixture } from '@/test/fixtures'
import { renderApp } from '@/test/render-app'

const PROJECT_ID = '5b1c1c4e-0000-4000-8000-000000000001'
const AUDIT_ID = '5b1c1c4e-0000-4000-8000-000000000002'

const watchesFixture: WatchesResponse = {
  mode: 'projects',
  generated_at: '2026-06-01T12:00:00Z',
  watches: [
    {
      uuid: 'u1',
      projectId: PROJECT_ID,
      name: 'Smlouvy',
      path: 'smlouvy/2026',
      added_on: '2026-06-01T12:00:00Z',
      documents: 3,
      pending_harvest: 1,
      legislation: 2,
      unconfirmed_changes: 1,
      last_check_at: '2026-06-02T06:00:00Z',
    },
  ],
  settings: { email: true, inApp: true, recipients: [] },
}

const emptyWatches: WatchesResponse = {
  mode: 'projects',
  generated_at: '2026-06-01T12:00:00Z',
  watches: [],
  settings: { email: true, inApp: true, recipients: [] },
}

const availableFixture: AvailableProjectsResponse = {
  mode: 'available',
  generated_at: '2026-06-01T12:00:00Z',
  projects: [{ projectId: AUDIT_ID, name: 'Audit' }],
}

/** The Audit project holds a `faktury` folder at its root and nothing below it. */
function browseFixture(path: string | null): BrowseResponse {
  return {
    mode: 'browse',
    generated_at: '2026-06-01T12:00:00Z',
    project: AUDIT_ID,
    name: 'Audit',
    path: path ?? '.',
    parent: path ? '.' : null,
    entries: path ? [] : [{ name: 'faktury', path: 'faktury' }],
  }
}

/** Watches for this screen, overview for the act badge the rail fetches itself. */
function stubWatches(
  payload: WatchesResponse,
  overview: OverviewResponse = overviewFixture,
  available: AvailableProjectsResponse = availableFixture,
) {
  const fetchMock = vi.fn().mockImplementation((input: unknown, init?: RequestInit) => {
    const url = new URL(String(input), 'http://localhost/')
    if (init?.method === 'POST') {
      return Promise.resolve({ ok: true, json: () => Promise.resolve({ ok: true }) })
    }
    const view = url.searchParams.get('view')
    const body =
      view === 'projects'
        ? payload
        : view === 'available'
          ? available
          : view === 'browse'
            ? browseFixture(url.searchParams.get('path'))
            : overview
    return Promise.resolve({ ok: true, json: () => Promise.resolve(body) })
  })
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

function renderHome(props: Partial<React.ComponentProps<typeof ProjectsHome>> = {}) {
  return renderApp(
    <ProjectsHome
      section="projects"
      onChangeSection={vi.fn()}
      onSelectWatch={vi.fn()}
      onOpenSettings={vi.fn()}
      {...props}
    />,
    { searchParams: { lang: 'en' } },
  )
}

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('ProjectsHome', () => {
  it('renders a card per watched folder', async () => {
    stubWatches(watchesFixture)
    renderHome()

    await waitFor(() => {
      expect(screen.getByText('Smlouvy')).toBeInTheDocument()
    })
    expect(screen.getByText('/smlouvy/2026')).toBeInTheDocument()
    expect(screen.getByText('3 documents')).toBeInTheDocument()
    expect(screen.getByText('2 acts')).toBeInTheDocument()
  })

  it('shows the empty landing when nothing is watched', async () => {
    stubWatches(emptyWatches)
    renderHome()

    await waitFor(() => {
      expect(screen.getByText('Watch the legislation in your projects')).toBeInTheDocument()
    })
  })

  it('keeps the app identity in the header across sections', async () => {
    stubWatches(watchesFixture)
    renderHome()

    expect(await screen.findByRole('heading', { name: 'Followed documents' })).toBeInTheDocument()
  })

  it('switches back to the act watchdog from the section navigation', async () => {
    stubWatches(watchesFixture)
    const onChangeSection = vi.fn()
    const user = userEvent.setup()
    renderHome({ onChangeSection })

    const projects = await screen.findByRole('button', { name: /Projects/ })
    expect(projects).toHaveAttribute('aria-current', 'page')

    await user.click(screen.getByRole('button', { name: /Acts/ }))
    expect(onChangeSection).toHaveBeenCalledWith('documents')
  })

  it('badges the act watchdog with the changes waiting there', async () => {
    stubWatches(watchesFixture)
    renderHome()

    // the overview fixture carries two unconfirmed act changes
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Acts/ })).toHaveTextContent('2')
    })
  })

  it('selects a watch when its card is clicked', async () => {
    stubWatches(watchesFixture)
    const onSelectWatch = vi.fn()
    const user = userEvent.setup()
    renderHome({ onSelectWatch })

    await waitFor(() => expect(screen.getByText('Smlouvy')).toBeInTheDocument())
    await user.click(screen.getByText('Smlouvy'))
    expect(onSelectWatch).toHaveBeenCalledWith('u1')
  })

  it('picks a project, then a folder inside it, and adds that folder', async () => {
    const fetchMock = stubWatches(watchesFixture)
    const user = userEvent.setup()
    renderHome()

    await user.click(await screen.findByRole('button', { name: /Add folder/ }))
    await user.click(await screen.findByRole('button', { name: /Audit/ }))

    // inside the project: its root is browsable and offers the `faktury` folder
    expect(await screen.findByText('Choose a folder to watch')).toBeInTheDocument()
    await user.click(await screen.findByRole('button', { name: /faktury/ }))
    await screen.findByText(/No sub-folders/)
    await user.click(screen.getByRole('button', { name: 'Watch this folder' }))

    await waitFor(() => {
      const posted = fetchMock.mock.calls
        .map(([, init]) => init as RequestInit | undefined)
        .filter((init) => init?.method === 'POST')
        .map((init) => JSON.parse(String(init?.body)))
      expect(posted).toEqual([{ action: 'project_add', project: AUDIT_ID, path: 'faktury' }])
    })
  })

  it('can watch the project root itself', async () => {
    const fetchMock = stubWatches(watchesFixture)
    const user = userEvent.setup()
    renderHome()

    await user.click(await screen.findByRole('button', { name: /Add folder/ }))
    await user.click(await screen.findByRole('button', { name: /Audit/ }))
    await screen.findByRole('button', { name: /faktury/ })
    await user.click(screen.getByRole('button', { name: 'Watch this folder' }))

    await waitFor(() => {
      const posted = fetchMock.mock.calls
        .map(([, init]) => init as RequestInit | undefined)
        .filter((init) => init?.method === 'POST')
        .map((init) => JSON.parse(String(init?.body)))
      expect(posted).toEqual([{ action: 'project_add', project: AUDIT_ID, path: '.' }])
    })
  })

  it('tells the user when there is no project to pick from', async () => {
    stubWatches(watchesFixture, overviewFixture, { ...availableFixture, projects: [] })
    const user = userEvent.setup()
    renderHome()

    await user.click(await screen.findByRole('button', { name: /Add folder/ }))
    expect(await screen.findByText(/No project yet/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Watch this folder' })).toBeDisabled()
  })
})
