import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { apiGet, CliError } from "./http"
import { EXIT } from "./config"

function fakeFetch(status: number, body: unknown): typeof fetch {
  return (async () =>
    new Response(typeof body === "string" ? body : JSON.stringify(body), {
      status,
      headers: { "content-type": "application/json" },
    })) as unknown as typeof fetch
}

beforeEach(() => {
  const dir = mkdtempSync(join(tmpdir(), "sid-http-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=sid_test\nSID_API_URL=http://sid.local\n", { mode: 0o600 })
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

describe("apiGet", () => {
  it("posílá x-api-key a vrací JSON", async () => {
    let seen: Record<string, string> = {}
    const f = (async (url: unknown, init: RequestInit) => {
      seen = Object.fromEntries(new Headers(init.headers))
      expect(String(url)).toBe("http://sid.local/api/v1/users/me")
      return new Response(JSON.stringify({ id: "u1" }), { status: 200 })
    }) as unknown as typeof fetch
    const out = await apiGet("/api/v1/users/me", f)
    expect(out).toEqual({ id: "u1" })
    expect(seen["x-api-key"]).toBe("sid_test")
  })

  it("bez klíče → CliError AUTH s návodem na /profile/api-klice, fetch se nevolá", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_URL=http://sid.local\n")
    let called = false
    const f = (async () => {
      called = true
      return new Response("{}")
    }) as unknown as typeof fetch
    const err = await apiGet("/api/search?q=x", f).catch((e: unknown) => e)
    expect(err).toBeInstanceOf(CliError)
    expect((err as CliError).exitCode).toBe(EXIT.AUTH)
    expect((err as CliError).message).toContain("/profile/api-klice")
    expect(called).toBe(false)
  })

  it("401 → AUTH, 404 → NOT_FOUND, 429 → RATE_LIMIT, 500 → ERROR", async () => {
    const cases = [
      [401, EXIT.AUTH],
      [404, EXIT.NOT_FOUND],
      [429, EXIT.RATE_LIMIT],
      [500, EXIT.ERROR],
    ] as const
    for (const [status, code] of cases) {
      const err = await apiGet("/api/x", fakeFetch(status, { error: { code: "X" } })).catch((e: unknown) => e)
      expect((err as CliError).exitCode).toBe(code)
    }
  })

  it("chybová hláška nikdy neobsahuje klíč", async () => {
    const err = await apiGet("/api/x", fakeFetch(500, "boom")).catch((e: unknown) => e)
    expect(String((err as CliError).message)).not.toContain("sid_test")
  })

  it("200 s ne-JSON tělem → srozumitelná chyba, ne syrové 'Failed to parse JSON'", async () => {
    // Real case: a wrong base URL returns an HTML login/proxy page with status 200.
    const f = (async () => new Response("<html><body>Login</body></html>", { status: 200 })) as unknown as typeof fetch
    const err = (await apiGet("/api/v1/users/me", f).catch((e: unknown) => e)) as CliError
    expect(err).toBeInstanceOf(CliError)
    expect(err.exitCode).toBe(EXIT.ERROR)
    expect(err.message).toContain("není JSON")
    expect(err.message).toContain("http://sid.local")
  })

  it("síťová chyba → ERROR, ne pád", async () => {
    const f = (async () => {
      throw new Error("ECONNREFUSED")
    }) as unknown as typeof fetch
    const err = await apiGet("/api/x", f).catch((e: unknown) => e)
    expect(err).toBeInstanceOf(CliError)
    expect((err as CliError).exitCode).toBe(EXIT.ERROR)
  })
})


describe("timeouty", () => {
  it("každý požadavek nese AbortSignal (jinak může cron viset navždy)", async () => {
    let sawSignal: unknown = null
    const f = (async (_u: unknown, init: RequestInit) => {
      sawSignal = init.signal
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await apiGet("/api/x", f)
    expect(sawSignal).toBeInstanceOf(AbortSignal)
  })

  it("vypršení timeoutu se hlásí srozumitelně, ne syrovým AbortError", async () => {
    const f = (async () => {
      const e = new Error("The operation was aborted due to timeout")
      e.name = "TimeoutError"
      throw e
    }) as unknown as typeof fetch
    const err = (await apiGet("/api/x", f).catch((e: unknown) => e)) as CliError
    expect(err).toBeInstanceOf(CliError)
    expect(err.message).toMatch(/vypršel|timeout/i)
  })
})

describe("download timeout", () => {
  it("apiRaw nesmí useknout dlouhý přenos JSON timeoutem", async () => {
    // AbortSignal.timeout also aborts the body, so a large document on a slow
    // link would die after 30s. Downloads get their own, much larger ceiling.
    const { DOWNLOAD_TIMEOUT_MS, REQUEST_TIMEOUT_MS } = await import("./http")
    expect(DOWNLOAD_TIMEOUT_MS).toBeGreaterThan(REQUEST_TIMEOUT_MS)
  })

  it("apiRaw s krátkým stropem přenos přeruší (parametr funguje)", async () => {
    const { apiRaw } = await import("./http")
    const f = (async (_u: unknown, init: RequestInit) =>
      new Promise((_res, rej) => {
        init.signal?.addEventListener("abort", () => {
          const e = new Error("aborted")
          e.name = "TimeoutError"
          rej(e)
        })
      })) as unknown as typeof fetch
    const err = (await apiRaw("/api/files/download/x", f, 50).catch((e: unknown) => e)) as CliError
    expect(err).toBeInstanceOf(CliError)
    expect(err.message).toMatch(/vypršel/i)
  })
})
