import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fetchDetail, fetchOverview } from '@/lib/api'
import { detailFixture, overviewFixture } from '@/test/fixtures'

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('fetchOverview', () => {
  it('returns parsed overview on success', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(overviewFixture),
      }),
    )

    const result = await fetchOverview()
    expect(result.mode).toBe('overview')
    expect(result.tracked_documents).toHaveLength(3)
  })

  it('throws on non-recoverable HTTP error', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 404,
        statusText: 'Not Found',
      }),
    )

    await expect(fetchOverview()).rejects.toThrow('HTTP 404')
  })

  it('falls back to local sample data on HTTP 500', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValueOnce({
          ok: false,
          status: 500,
          statusText: 'Internal Server Error',
        })
        .mockResolvedValueOnce({
          ok: true,
          json: () =>
            Promise.resolve({
              trackedDocuments: [
                {
                  uuid: 'sample-1',
                  codexisId: 'cdx://doc/SAMPLE',
                  name: 'Sample',
                  added_on: '2026-01-01T00:00:00Z',
                  tracking_type: 'all',
                  parts: [],
                  changes: [
                    {
                      source_documents: [],
                      detected_on: '2026-01-01T00:00:00Z',
                      effective_on: '2026-01-01',
                      change_type: 'document_change',
                      description_md: 'x',
                      confirmed_on: null,
                    },
                  ],
                },
              ],
            }),
        }),
    )

    const result = await fetchOverview()
    expect(result.mode).toBe('overview')
    expect(result.tracked_documents[0].uuid).toBe('sample-1')
  })

  it('throws on invalid response shape', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ invalid: true }),
      }),
    )

    await expect(fetchOverview()).rejects.toThrow()
  })
})

describe('fetchDetail', () => {
  it('returns parsed detail on success', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(detailFixture),
      }),
    )

    const result = await fetchDetail('2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d')
    expect(result.mode).toBe('detail')
    expect(result.document.uuid).toBe('2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d')
  })

  it('includes uuid in query string', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(detailFixture),
    })
    vi.stubGlobal('fetch', mockFetch)

    await fetchDetail('test-uuid')
    const calledUrl = mockFetch.mock.calls[0][0] as string
    expect(calledUrl).toContain('uuid=test-uuid')
  })

  it('falls back to local sample data detail on HTTP 500', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValueOnce({
          ok: false,
          status: 500,
          statusText: 'Internal Server Error',
        })
        .mockResolvedValueOnce({
          ok: true,
          json: () =>
            Promise.resolve({
              trackedDocuments: [
                {
                  uuid: 'fallback-uuid',
                  codexisId: 'cdx://doc/SAMPLE',
                  name: 'Sample',
                  added_on: '2026-01-01T00:00:00Z',
                  tracking_type: 'all',
                  parts: [],
                  changes: [
                    {
                      source_documents: [],
                      detected_on: '2026-01-01T00:00:00Z',
                      effective_on: '2026-01-01',
                      change_type: 'document_change',
                      description_md: 'x',
                      confirmed_on: null,
                    },
                  ],
                },
              ],
            }),
        }),
    )

    const result = await fetchDetail('fallback-uuid')
    expect(result.mode).toBe('detail')
    expect(result.document.uuid).toBe('fallback-uuid')
  })
})
