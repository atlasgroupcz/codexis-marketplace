import { afterEach, describe, expect, it, vi } from 'vitest'
import { z } from 'zod'
import { createApiClient } from './api-client'

const base = () => 'http://localhost/api/'
const okJson = (body: unknown) =>
  Promise.resolve({ ok: true, status: 200, statusText: 'OK', json: () => Promise.resolve(body) } as Response)

afterEach(() => vi.unstubAllGlobals())

describe('createApiClient', () => {
  it('getJson parses the response with the schema', async () => {
    vi.stubGlobal('fetch', vi.fn(() => okJson({ n: 1 })))
    const client = createApiClient(base)
    const out = await client.getJson(z.object({ n: z.number() }))
    expect(out).toEqual({ n: 1 })
  })

  it('getJson throws on non-ok HTTP', async () => {
    vi.stubGlobal('fetch', vi.fn(() =>
      Promise.resolve({ ok: false, status: 500, statusText: 'Err', json: () => Promise.resolve({}) } as Response)))
    const client = createApiClient(base)
    await expect(client.getJson(z.object({}))).rejects.toThrow('HTTP 500')
  })

  it('postJson rejects on {ok:false} returned with HTTP 200', async () => {
    vi.stubGlobal('fetch', vi.fn(() => okJson({ ok: false, error: 'nope' })))
    const client = createApiClient(base)
    await expect(client.postJson('do', z.object({}))).rejects.toThrow('nope')
  })

  it('surfaces the JSON error body on a non-ok HTTP response', async () => {
    vi.stubGlobal('fetch', vi.fn(() =>
      Promise.resolve({
        ok: false,
        status: 409,
        statusText: 'Conflict',
        json: () => Promise.resolve({ ok: false, error: 'už sledované' }),
      } as Response)))
    const client = createApiClient(base)
    await expect(client.postJson('add', z.object({}))).rejects.toThrow('už sledované')
  })

  it('postJson sends {action, ...payload}', async () => {
    const fetchMock = vi.fn<typeof fetch>(() => okJson({ ok: true }))
    vi.stubGlobal('fetch', fetchMock)
    const client = createApiClient(base)
    await client.postJson('save', z.object({ ok: z.boolean() }), { x: 2 })
    const body = JSON.parse((fetchMock.mock.calls[0]![1] as RequestInit).body as string)
    expect(body).toEqual({ action: 'save', x: 2 })
  })
})
