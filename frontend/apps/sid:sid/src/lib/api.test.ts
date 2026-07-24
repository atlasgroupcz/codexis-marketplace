// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { deleteApiKey, fetchStatus, saveApiKey, saveNotifySettings } from './api'

const STATUS = {
  ok: true,
  configured: true,
  baseUrl: 'https://sid.test',
  start: 'sid_abc',
  userEmail: 'ja@firma.cz',
  notify: { email: false, inApp: true, recipients: [] },
}

function stubFetch(payload: unknown, ok = true) {
  const mock = vi.fn().mockResolvedValue({
    ok,
    status: ok ? 200 : 500,
    statusText: ok ? 'OK' : 'Server Error',
    json: () => Promise.resolve(payload),
  })
  vi.stubGlobal('fetch', mock)
  return mock
}

function bodyOf(mock: ReturnType<typeof vi.fn>): Record<string, unknown> {
  return JSON.parse(String(mock.mock.calls[0]![1].body))
}

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('fetchStatus', () => {
  it('asks for the status and returns it parsed', async () => {
    const mock = stubFetch(STATUS)
    const status = await fetchStatus()
    expect(bodyOf(mock)).toEqual({ action: 'get_status' })
    expect(status.start).toBe('sid_abc')
    expect(status.notify.inApp).toBe(true)
  })

  it('rejects a response that does not match the schema', async () => {
    // Guards against the panel rendering `undefined` when the CGI contract
    // drifts - zod should refuse it rather than let it through.
    stubFetch({ ok: true, configured: 'yes' })
    await expect(fetchStatus()).rejects.toThrow()
  })

  it('turns an HTTP failure into an error, not a silent empty state', async () => {
    stubFetch({}, false)
    await expect(fetchStatus()).rejects.toThrow(/500/)
  })
})

describe('write actions', () => {
  it('saveApiKey sends the key under api_key', async () => {
    const mock = stubFetch({ ok: true, verified: true, start: 'sid_abc' })
    await saveApiKey('sid_abc123')
    expect(bodyOf(mock)).toEqual({ action: 'save_config', api_key: 'sid_abc123' })
  })

  it('a rejected key comes back as ok:false rather than throwing', async () => {
    // The panel shows the reason inline; only transport failures throw.
    stubFetch({ ok: false, error: 'SID klíč odmítl' })
    await expect(saveApiKey('bad')).resolves.toMatchObject({ ok: false })
  })

  it('deleteApiKey sends just the action', async () => {
    const mock = stubFetch({ ok: true })
    await deleteApiKey()
    expect(bodyOf(mock)).toEqual({ action: 'delete_config' })
  })

  it('saveNotifySettings sends every channel field', async () => {
    const mock = stubFetch({ ok: true, notify: STATUS.notify })
    await saveNotifySettings({ email: true, inApp: false, recipients: ['sef@firma.cz'] })
    expect(bodyOf(mock)).toEqual({
      action: 'save_notify_settings',
      email: true,
      inApp: false,
      recipients: ['sef@firma.cz'],
    })
  })
})

describe('endpoint', () => {
  it('posts to the component-relative api path, never an absolute SID url', async () => {
    // In the VM this resolves against the component directory, so it reaches
    // the CGI binary; the dev proxy adds a trailing slash, hence the optional /.
    const mock = stubFetch(STATUS)
    await fetchStatus()
    const url = new URL(String(mock.mock.calls[0]![0]))
    expect(url.pathname).toMatch(/\/api\/?$/)
    expect(url.origin).toBe(window.location.origin)
    expect(mock.mock.calls[0]![1].method).toBe('POST')
  })
})
