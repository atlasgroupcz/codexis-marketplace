import { describe, expect, it } from 'vitest'
import {
  browseResponseSchema,
  folderResponseSchema,
  foldersResponseSchema,
} from '@/lib/schemas'

describe('folder schemas', () => {
  it('parses a folders overview response', () => {
    const parsed = foldersResponseSchema.parse({
      mode: 'folders',
      generated_at: '2026-06-01T12:00:00Z',
      folders: [
        {
          root: '/home/u/c',
          name: 'C',
          documents: 2,
          pending_harvest: 0,
          legislation: 1,
          unconfirmed_changes: 0,
        },
      ],
      settings: { email: true, inApp: false },
    })
    expect(parsed.folders[0].name).toBe('C')
    // recipients defaults to an empty array
    expect(parsed.settings.recipients).toEqual([])
  })

  it('parses a folder detail and applies legislation/changes defaults', () => {
    const parsed = folderResponseSchema.parse({
      mode: 'folder',
      generated_at: '2026-06-01T12:00:00Z',
      folder: {
        root: '/home/u/c',
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
    expect(parsed.folder.documents[0].legislation).toEqual([])
    // tracked legislation defaults
    expect(parsed.folder.legislation[0].changes).toEqual([])
    expect(parsed.folder.legislation[0].tracked).toBe(false)
    expect(parsed.folder.legislation[0].unconfirmed_changes).toBe(0)
  })

  it('parses a browse response', () => {
    const parsed = browseResponseSchema.parse({
      mode: 'browse',
      generated_at: '2026-06-01T12:00:00Z',
      path: '/home/u',
      parent: null,
      home: '/home/u',
      entries: [{ name: 'docs', path: '/home/u/docs', isDir: true }],
    })
    expect(parsed.entries[0].name).toBe('docs')
    expect(parsed.parent).toBeNull()
  })
})
