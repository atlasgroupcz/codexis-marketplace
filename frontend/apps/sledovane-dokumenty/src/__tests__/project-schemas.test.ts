import { describe, expect, it } from 'vitest'
import {
  availableProjectsResponseSchema,
  notifyResponseSchema,
  projectResponseSchema,
  projectsResponseSchema,
} from '@/lib/schemas'

describe('project schemas', () => {
  it('parses a projects overview response', () => {
    const parsed = projectsResponseSchema.parse({
      mode: 'projects',
      generated_at: '2026-06-01T12:00:00Z',
      projects: [
        {
          projectId: 'p1',
          name: 'C',
          documents: 2,
          pending_harvest: 0,
          legislation: 1,
          unconfirmed_changes: 0,
        },
      ],
      settings: { email: true, inApp: false },
    })
    expect(parsed.projects[0]?.name).toBe('C')
    // recipients defaults to an empty array
    expect(parsed.settings.recipients).toEqual([])
  })

  it('parses a project detail and applies legislation/changes defaults', () => {
    const parsed = projectResponseSchema.parse({
      mode: 'project',
      generated_at: '2026-06-01T12:00:00Z',
      project: {
        projectId: 'p1',
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
    expect(parsed.project.documents[0]?.legislation).toEqual([])
    // tracked legislation defaults
    expect(parsed.project.legislation[0]?.changes).toEqual([])
    expect(parsed.project.legislation[0]?.tracked).toBe(false)
    expect(parsed.project.legislation[0]?.unconfirmed_changes).toBe(0)
  })

  it('parses the projects still available to watch', () => {
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
