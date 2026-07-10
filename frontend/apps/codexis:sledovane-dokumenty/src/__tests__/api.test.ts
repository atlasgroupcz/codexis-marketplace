import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fetchDetail, fetchNotify, fetchOverview, saveNotify } from '@/lib/api'
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

  it('propagates HTTP 500 as error', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
      }),
    )

    await expect(fetchOverview()).rejects.toThrow('HTTP 500')
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

  it('propagates HTTP 500 as error', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
      }),
    )

    await expect(fetchDetail('any-uuid')).rejects.toThrow('HTTP 500')
  })
})

describe('fetchNotify', () => {
  it('returns parsed notify settings and userEmail on success', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () =>
          Promise.resolve({
            mode: 'notify',
            settings: { email: true, inApp: false, recipients: ['a@b.cz'] },
            userEmail: 'me@example.com',
          }),
      }),
    )

    const result = await fetchNotify()
    expect(result.userEmail).toBe('me@example.com')
    expect(result.settings).toEqual({ email: true, inApp: false, recipients: ['a@b.cz'] })
  })

  it('requests the notify view', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({
          mode: 'notify',
          settings: { email: true, inApp: true, recipients: [] },
          userEmail: '',
        }),
    })
    vi.stubGlobal('fetch', mockFetch)

    await fetchNotify()
    const calledUrl = mockFetch.mock.calls[0][0] as string
    expect(calledUrl).toContain('view=notify')
  })

  it('propagates HTTP errors', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({ ok: false, status: 500, statusText: 'Internal Server Error' }),
    )

    await expect(fetchNotify()).rejects.toThrow('HTTP 500')
  })
})

describe('saveNotify', () => {
  it('posts save_notify_settings with the given values', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ ok: true }),
    })
    vi.stubGlobal('fetch', mockFetch)

    const result = await saveNotify({ email: true, inApp: false, recipients: ['x@y.cz'] })
    expect(result.ok).toBe(true)

    const body = JSON.parse((mockFetch.mock.calls[0][1] as RequestInit).body as string) as Record<
      string,
      unknown
    >
    expect(body).toEqual({
      action: 'save_notify_settings',
      email: true,
      inApp: false,
      recipients: ['x@y.cz'],
    })
  })

  it('propagates HTTP errors', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({ ok: false, status: 400, statusText: 'Bad Request' }),
    )

    await expect(saveNotify({ email: true, inApp: true, recipients: [] })).rejects.toThrow('HTTP 400')
  })
})
