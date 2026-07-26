// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fetchNotify, saveNotify } from './api'

/** The fetch call under test, so a missing call fails as a missing call. */
function firstCall(mock: { mock: { calls: Array<Array<unknown>> } }): Array<unknown> {
  const call = mock.mock.calls[0]
  if (!call) throw new Error('fetch was never called')
  return call
}


beforeEach(() => {
  vi.restoreAllMocks()
})

describe('fetchNotify', () => {
  it('returns parsed settings + userEmail on success', async () => {
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
    expect(result.settings).toEqual({ email: true, inApp: false, recipients: ['a@b.cz'] })
    expect(result.userEmail).toBe('me@example.com')
  })

  it('requests the notify view', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({
          mode: 'notify',
          settings: { email: false, inApp: true, recipients: [] },
          userEmail: '',
        }),
    })
    vi.stubGlobal('fetch', mockFetch)

    await fetchNotify()
    const calledUrl = firstCall(mockFetch)[0] as string
    expect(calledUrl).toContain('view=notify')
  })

  it('throws on invalid response shape', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ invalid: true }),
      }),
    )

    await expect(fetchNotify()).rejects.toThrow()
  })

  it('throws on non-recoverable HTTP error', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: () => Promise.resolve(null),
      }),
    )

    await expect(fetchNotify()).rejects.toThrow('HTTP 500')
  })
})

describe('saveNotify', () => {
  it('posts the save_notify_settings action with the given values', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ ok: true, settings: { email: true, inApp: true, recipients: [] } }),
    })
    vi.stubGlobal('fetch', mockFetch)

    await saveNotify({ email: true, inApp: true, recipients: ['x@y.cz'] })

    const [, init] = mockFetch.mock.calls[0] as [string, RequestInit]
    const body = JSON.parse(init.body as string) as Record<string, unknown>
    expect(body).toEqual({
      action: 'save_notify_settings',
      email: true,
      inApp: true,
      recipients: ['x@y.cz'],
    })
  })

  it('rejects on {ok: false} even with HTTP 200', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ ok: false, error: 'Neplatná e-mailová adresa' }),
      }),
    )

    await expect(saveNotify({ email: true, inApp: false, recipients: ['bad'] })).rejects.toThrow(
      'Neplatná e-mailová adresa',
    )
  })

  it('propagates HTTP 400 as error', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 400,
        statusText: 'Bad Request',
        json: () => Promise.resolve({ error: 'Neplatná e-mailová adresa' }),
      }),
    )

    await expect(saveNotify({ email: true, inApp: false, recipients: ['bad'] })).rejects.toThrow(
      'Neplatná e-mailová adresa',
    )
  })
})
