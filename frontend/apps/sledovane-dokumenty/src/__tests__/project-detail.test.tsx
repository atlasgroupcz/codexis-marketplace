import { screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ProjectDetail } from '@/components/project-detail'
import type { TrackedLegislation, WatchResponse } from '@/lib/schemas'
import { renderApp } from '@/test/render-app'

function stubWatch(payload: WatchResponse) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve(payload) }),
  )
}

function watchWith(legislation: Array<TrackedLegislation>): WatchResponse {
  return {
    mode: 'watch',
    generated_at: '2026-06-01T12:00:00Z',
    watch: {
      uuid: 'u1',
      projectId: '5b1c1c4e-0000-4000-8000-000000000001',
      path: 'smlouvy',
      name: 'Smlouvy',
      added_on: '2026-06-01T12:00:00Z',
      documents: [],
      documents_count: 0,
      pending_harvest: 0,
      legislation,
      last_check_at: null,
    },
  }
}

function renderWatch() {
  return renderApp(<ProjectDetail watchId="u1" onBack={vi.fn()} onOpenSettings={vi.fn()} />, {
    searchParams: { lang: 'en' },
  })
}

const resolvedAct: TrackedLegislation = {
  text: 'Občanský zákoník (89/2012 Sb.)',
  uri: 'https://www.zakonyprolidi.cz/cs/2012-89',
  codexisId: 'CR26785',
  tracked: true,
  changes: [],
  unconfirmed_changes: 0,
}

const unresolvedAct: TrackedLegislation = {
  text: 'Neznámý předpis',
  uri: 'https://www.zakonyprolidi.cz/cs/2012-89',
  codexisId: null,
  tracked: false,
  changes: [],
  unconfirmed_changes: 0,
}

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('ProjectDetail legislation link', () => {
  it('links a resolved act to its CODEXIS document', async () => {
    stubWatch(watchWith([resolvedAct]))
    renderWatch()

    const link = await screen.findByRole('link')
    expect(link).toHaveAttribute('href', 'https://next.codexis.cz/doc/CR26785')
  })

  it('never links legislation to zakonyprolidi', async () => {
    stubWatch(watchWith([resolvedAct]))
    renderWatch()

    await screen.findByText(resolvedAct.text)
    const hrefs = screen.queryAllByRole('link').map((a) => a.getAttribute('href') ?? '')
    expect(hrefs.some((h) => h.includes('zakonyprolidi'))).toBe(false)
  })

  it('renders no external link for an act without a CODEXIS id', async () => {
    stubWatch(watchWith([unresolvedAct]))
    renderWatch()

    await screen.findByText(unresolvedAct.text)
    expect(screen.queryAllByRole('link')).toHaveLength(0)
  })

  it('names the watched folder inside the project', async () => {
    stubWatch(watchWith([resolvedAct]))
    renderWatch()

    expect(await screen.findByRole('heading', { name: 'Smlouvy' })).toBeInTheDocument()
    expect(screen.getByText('/smlouvy')).toBeInTheDocument()
  })
})
