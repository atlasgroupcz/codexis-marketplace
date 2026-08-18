import { screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FolderDetail } from '@/components/folder-detail'
import type { FolderResponse, TrackedLegislation } from '@/lib/schemas'
import { renderApp } from '@/test/render-app'

function stubFolder(payload: FolderResponse) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve(payload) }),
  )
}

function folderWith(legislation: Array<TrackedLegislation>): FolderResponse {
  return {
    mode: 'folder',
    generated_at: '2026-06-01T12:00:00Z',
    folder: {
      root: '/home/u/my-contracts',
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

function renderFolder() {
  return renderApp(
    <FolderDetail root="/home/u/my-contracts" onBack={vi.fn()} onOpenSettings={vi.fn()} />,
    { searchParams: { lang: 'en' } },
  )
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

describe('FolderDetail legislation link', () => {
  it('links a resolved act to its CODEXIS document', async () => {
    stubFolder(folderWith([resolvedAct]))
    renderFolder()

    const link = await screen.findByRole('link')
    expect(link).toHaveAttribute('href', 'https://next.codexis.cz/doc/CR26785')
  })

  it('never links legislation to zakonyprolidi', async () => {
    stubFolder(folderWith([resolvedAct]))
    renderFolder()

    await screen.findByText(resolvedAct.text)
    const hrefs = screen.queryAllByRole('link').map((a) => a.getAttribute('href') ?? '')
    expect(hrefs.some((h) => h.includes('zakonyprolidi'))).toBe(false)
  })

  it('renders no external link for an act without a CODEXIS id', async () => {
    stubFolder(folderWith([unresolvedAct]))
    renderFolder()

    await screen.findByText(unresolvedAct.text)
    expect(screen.queryAllByRole('link')).toHaveLength(0)
  })
})
