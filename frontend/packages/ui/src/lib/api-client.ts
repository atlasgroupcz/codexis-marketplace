import type { z } from 'zod'

export interface ApiClient {
  buildUrl: (params?: Record<string, string>) => string
  getJson: <T>(schema: z.ZodType<T>, params?: Record<string, string>) => Promise<T>
  postJson: <T>(action: string, schema: z.ZodType<T>, payload?: Record<string, unknown>) => Promise<T>
}

function defaultBase(): string {
  // This library package carries no Vite ambient types (unlike the apps that
  // consume it), so `import.meta.env` is read via a narrow inline cast rather
  // than relying on `vite/client` typings.
  const env = (import.meta as { env?: { DEV?: boolean } }).env
  return env?.DEV ? '/api/' : './api'
}

/** The message from a non-ok response: the JSON body's `error` field when the
 * server sent one, otherwise the bare HTTP status. Watcher CGIs answer errors as
 * `{ ok: false, error: "<message>" }`, so surfacing `error` shows the real cause
 * (e.g. "IČO už je sledované") instead of a naked "HTTP 409". */
function httpError(res: Response, data: unknown): Error {
  const err =
    data && typeof data === 'object' && 'error' in data ? (data as { error: unknown }).error : undefined
  return new Error(typeof err === 'string' && err ? err : `HTTP ${res.status}: ${res.statusText}`)
}

export function createApiClient(base: () => string = defaultBase): ApiClient {
  function buildUrl(params?: Record<string, string>): string {
    const url = new URL(base(), window.location.href)
    if (params) for (const [k, v] of Object.entries(params)) url.searchParams.set(k, v)
    return url.toString()
  }

  async function getJson<T>(schema: z.ZodType<T>, params?: Record<string, string>): Promise<T> {
    const res = await fetch(buildUrl(params), { headers: { Accept: 'application/json' }, cache: 'no-store' })
    const data: unknown = await res.json().catch(() => null)
    if (!res.ok) throw httpError(res, data)
    return schema.parse(data)
  }

  async function postJson<T>(action: string, schema: z.ZodType<T>, payload?: Record<string, unknown>): Promise<T> {
    const res = await fetch(buildUrl(), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
      body: JSON.stringify({ action, ...payload }),
    })
    const data: unknown = await res.json().catch(() => null)
    if (!res.ok) throw httpError(res, data)
    // Actions also signal failure via `{ ok: false }` on an HTTP 200. Reject on it
    // too, so every caller has one error channel: resolve = success, reject = error.
    if (data && typeof data === 'object' && 'ok' in data && (data as { ok: unknown }).ok === false) {
      const err = (data as { error?: unknown }).error
      throw new Error(typeof err === 'string' && err ? err : 'Request failed')
    }
    return schema.parse(data)
  }

  return { buildUrl, getJson, postJson }
}
