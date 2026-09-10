import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ProjectsHome } from '@/components/projects-home'
import type { AvailableProjectsResponse, OverviewResponse, ProjectsResponse } from '@/lib/schemas'
import { overviewFixture } from '@/test/fixtures'
import { renderApp } from '@/test/render-app'

const PROJECT_ID = '5b1c1c4e-0000-4000-8000-000000000001'

const projectsFixture: ProjectsResponse = {
  mode: 'projects',
  generated_at: '2026-06-01T12:00:00Z',
  projects: [
    {
      uuid: 'u1',
      projectId: PROJECT_ID,
      name: 'Smlouvy',
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

const emptyProjects: ProjectsResponse = {
  mode: 'projects',
  generated_at: '2026-06-01T12:00:00Z',
  projects: [],
  settings: { email: true, inApp: true, recipients: [] },
}

const availableFixture: AvailableProjectsResponse = {
  mode: 'available',
  generated_at: '2026-06-01T12:00:00Z',
  projects: [{ projectId: '5b1c1c4e-0000-4000-8000-000000000002', name: 'Audit' }],
}

/** Projects for this screen, overview for the act badge the rail fetches itself. */
function stubProjects(
  payload: ProjectsResponse,
  overview: OverviewResponse = overviewFixture,
  available: AvailableProjectsResponse = availableFixture,
) {
  const fetchMock = vi.fn().mockImplementation((input: unknown, init?: RequestInit) => {
    const url = String(input)
    if (init?.method === 'POST') {
      return Promise.resolve({ ok: true, json: () => Promise.resolve({ ok: true }) })
    }
    const body = url.includes('view=projects')
      ? payload
      : url.includes('view=available')
        ? available
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
      onSelectProject={vi.fn()}
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
  it('renders project cards from the overview', async () => {
    stubProjects(projectsFixture)
    renderHome()

    await waitFor(() => {
      expect(screen.getByText('Smlouvy')).toBeInTheDocument()
    })
    expect(screen.getByText('Last check 2026-06-02')).toBeInTheDocument()
    expect(screen.getByText('3 documents')).toBeInTheDocument()
    expect(screen.getByText('2 acts')).toBeInTheDocument()
  })

  it('shows the empty landing when no projects are watched', async () => {
    stubProjects(emptyProjects)
    renderHome()

    await waitFor(() => {
      expect(screen.getByText('Watch the legislation in your projects')).toBeInTheDocument()
    })
  })

  it('keeps the app identity in the header across sections', async () => {
    stubProjects(projectsFixture)
    renderHome()

    expect(await screen.findByRole('heading', { name: 'Followed documents' })).toBeInTheDocument()
  })

  it('switches back to the act watchdog from the section navigation', async () => {
    stubProjects(projectsFixture)
    const onChangeSection = vi.fn()
    const user = userEvent.setup()
    renderHome({ onChangeSection })

    const projects = await screen.findByRole('button', { name: /Projects/ })
    expect(projects).toHaveAttribute('aria-current', 'page')

    await user.click(screen.getByRole('button', { name: /Acts/ }))
    expect(onChangeSection).toHaveBeenCalledWith('documents')
  })

  it('badges the act watchdog with the changes waiting there', async () => {
    stubProjects(projectsFixture)
    renderHome()

    // the overview fixture carries two unconfirmed act changes
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Acts/ })).toHaveTextContent('2')
    })
  })

  it('selects a project when its card is clicked', async () => {
    stubProjects(projectsFixture)
    const onSelectProject = vi.fn()
    const user = userEvent.setup()
    renderHome({ onSelectProject })

    await waitFor(() => expect(screen.getByText('Smlouvy')).toBeInTheDocument())
    await user.click(screen.getByText('Smlouvy'))
    expect(onSelectProject).toHaveBeenCalledWith(PROJECT_ID)
  })

  it('offers the projects not yet watched and adds the chosen one', async () => {
    const fetchMock = stubProjects(projectsFixture)
    const user = userEvent.setup()
    renderHome()

    await user.click(await screen.findByRole('button', { name: /Add project/ }))
    const option = await screen.findByRole('button', { name: /Audit/ })
    await user.click(option)
    await user.click(screen.getByRole('button', { name: 'Watch this project' }))

    await waitFor(() => {
      const posted = fetchMock.mock.calls
        .map(([, init]) => init as RequestInit | undefined)
        .filter((init) => init?.method === 'POST')
        .map((init) => JSON.parse(String(init?.body)))
      expect(posted).toEqual([
        { action: 'project_add', project: '5b1c1c4e-0000-4000-8000-000000000002' },
      ])
    })
  })

  it('tells the user when every project is already watched', async () => {
    stubProjects(projectsFixture, overviewFixture, { ...availableFixture, projects: [] })
    const user = userEvent.setup()
    renderHome()

    await user.click(await screen.findByRole('button', { name: /Add project/ }))
    expect(await screen.findByText(/No other project to watch/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Watch this project' })).toBeDisabled()
  })
})
