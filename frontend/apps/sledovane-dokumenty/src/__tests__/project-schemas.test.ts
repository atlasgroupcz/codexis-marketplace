import { describe, expect, it } from 'vitest'
import {
  availableProjectsResponseSchema,
  browseResponseSchema,
  notifyResponseSchema,
  watchesResponseSchema,
  watchResponseSchema,
} from '@/lib/schemas'

describe('project schemas', () => {
  it('parses a projects overview response', () => {
    const parsed = watchesResponseSchema.parse({
      mode: 'projects',
      generated_at: '2026-06-01T12:00:00Z',
      watches: [
        {
          uuid: 'u1',
          projectId: 'p1',
          path: '.',
          name: 'C',
          documents: 2,
          pending_harvest: 0,
          legislation: 1,
          unconfirmed_changes: 0,
        },
      ],
      settings: { email: true, inApp: false },
    })
    expect(parsed.watches[0]?.name).toBe('C')
    // recipients defaults to an empty array
    expect(parsed.settings.recipients).toEqual([])
  })

  it('parses a project detail and applies legislation/changes defaults', () => {
    const parsed = watchResponseSchema.parse({
      mode: 'watch',
      generated_at: '2026-06-01T12:00:00Z',
      watch: {
        uuid: 'u1',
        projectId: 'p1',
        path: 'smlouvy',
        name: 'C',
        documents: [
          {
            path: './a.docx',
            sha256: 'abc',
            discoveredAt: '2026-06-01T12:00:00Z',
            extractedAt: null,
            updatedAt: '2026-06-01T12:00:00Z',
          },
        ],
        documents_count: 1,
        pending_harvest: 1,
        legislation: [
          { uri: 'https://www.zakonyprolidi.cz/cs/2012-89', text: 'OZ', codexisId: 'CR26785' },
        ],
        last_check_at: null,
      },
    })
    // document.legislation defaulted to []
    expect(parsed.watch.documents[0]?.legislation).toEqual([])
    // tracked legislation defaults
    expect(parsed.watch.legislation[0]?.changes).toEqual([])
    expect(parsed.watch.legislation[0]?.tracked).toBe(false)
    expect(parsed.watch.legislation[0]?.unconfirmed_changes).toBe(0)
  })

  it('parses a folder listing of a project', () => {
    const parsed = browseResponseSchema.parse({
      mode: 'browse',
      generated_at: '2026-06-01T12:00:00Z',
      project: 'p1',
      name: 'Smlouvy',
      path: 'smlouvy',
      parent: '.',
      entries: [{ name: '2026', path: 'smlouvy/2026' }],
    })
    expect(parsed.entries[0]?.path).toBe('smlouvy/2026')
    expect(parsed.parent).toBe('.')
  })

  it('parses the projects a folder can be picked from', () => {
    const parsed = availableProjectsResponseSchema.parse({
      mode: 'available',
      generated_at: '2026-06-01T12:00:00Z',
      projects: [{ projectId: 'p2', name: 'Audit' }],
    })
    expect(parsed.projects[0]?.name).toBe('Audit')
  })

  it('parses a notify response, including an empty userEmail outside a CDX daemon', () => {
    const parsed = notifyResponseSchema.parse({
      mode: 'notify',
      settings: { email: true, inApp: false },
      userEmail: '',
    })
    expect(parsed.settings.recipients).toEqual([])
    expect(parsed.userEmail).toBe('')
  })
})
