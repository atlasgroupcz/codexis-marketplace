import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { sendDigest, graphqlUrl } from "./notify"

let home: string
beforeEach(() => {
  home = mkdtempSync(join(tmpdir(), "sid-home-"))
  process.env.CODEXIS_PUBLIC_USER_HOME = home
  // CODEXIS_PUBLIC_DAEMON_URL is the GraphQL URL.
  process.env.CODEXIS_PUBLIC_DAEMON_URL = "http://daemon.local/graphql"
  process.env.SID_ENV_FILE = join(home, ".env")
  const defaults = join(home, "defaults.json")
  writeFileSync(defaults, JSON.stringify({ notify: { inApp: true } }))
  process.env.SID_DEFAULTS_FILE = defaults
})

function writeSecret(value: string): void {
  mkdirSync(join(home, ".cdx", "env"), { recursive: true })
  writeFileSync(join(home, ".cdx", "env", "secret"), value)
}

describe("URL odvození", () => {
  it("graphqlUrl /graphql nezdvojuje", () => {
    expect(graphqlUrl("http://d/graphql")).toBe("http://d/graphql")
    expect(graphqlUrl("http://d")).toBe("http://d/graphql")
  })
})

describe("sendDigest", () => {
  it("bez secret souboru vrátí false a NEvolá fetch", async () => {
    let called = false
    const f = (async () => {
      called = true
      return new Response("{}")
    }) as unknown as typeof fetch
    expect(await sendDigest("s", f)).toBe(false)
    expect(called).toBe(false)
  })

  it("posílá bell přes GraphQL createNotification s Bearer tokenem", async () => {
    writeSecret("vmtoken\n")
    const seen: { url?: string; auth?: string; body?: string } = {}
    const f = (async (u: unknown, init: RequestInit) => {
      seen.url = String(u)
      seen.auth = new Headers(init.headers).get("authorization") ?? ""
      seen.body = String(init.body)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch

    expect(await sendDigest("Předmět", f)).toBe(true)
    expect(seen.url).toBe("http://daemon.local/graphql")
    expect(seen.auth).toBe("Bearer vmtoken")
    expect(seen.body).toContain("createNotification")
    expect(JSON.parse(seen.body!).variables.input.message).toBe("Předmět")
  })

  it("normalizuje 'Authorization:' i holý token na Bearer", async () => {
    writeSecret("Authorization: Bearer abc\n")
    let auth = ""
    const f = (async (_u: unknown, init: RequestInit) => {
      auth = new Headers(init.headers).get("authorization") ?? ""
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", f)
    expect(auth).toBe("Bearer abc")
  })

  it("vypnutý bell → nic se nevolá", async () => {
    writeSecret("t")
    writeFileSync(process.env.SID_DEFAULTS_FILE!, JSON.stringify({ notify: { inApp: false } }))
    let called = false
    const f = (async () => {
      called = true
      return new Response("{}")
    }) as unknown as typeof fetch
    expect(await sendDigest("s", f)).toBe(false)
    expect(called).toBe(false)
  })

  it("HTTP chyba → false, nikdy nehází", async () => {
    writeSecret("t")
    const f = (async () => new Response("x", { status: 500 })) as unknown as typeof fetch
    expect(await sendDigest("s", f)).toBe(false)
  })

  it("výjimka transportu → false, nikdy nehází", async () => {
    writeSecret("t")
    const f = (async () => {
      throw new Error("ECONNREFUSED")
    }) as unknown as typeof fetch
    expect(await sendDigest("s", f)).toBe(false)
  })
})

describe("žádný e-mail", () => {
  /**
   * SID posílá vlastní e-maily (šablony, odkaz na konkrétní dokument,
   * license gating). Plugin proto mail neposílá vůbec - ani když v
   * nastavení po starší verzi zůstalo `email: true` a seznam adresátů.
   */
  it("nesahá na REST endpoint daemona ani se zděděným `email: true`", async () => {
    writeSecret("t")
    writeFileSync(
      process.env.SID_DEFAULTS_FILE!,
      JSON.stringify({ notify: { email: true, inApp: true, recipients: ["sef@firma.cz"] } }),
    )
    const urls: string[] = []
    const f = (async (u: unknown) => {
      urls.push(String(u))
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch

    await sendDigest("s", f)
    expect(urls).toEqual(["http://daemon.local/graphql"])
    expect(urls.some((u) => u.includes("/rest/"))).toBe(false)
  })

  it("neptá se daemona na adresu přihlášeného uživatele", async () => {
    writeSecret("t")
    let calls = 0
    const f = (async (_u: unknown, init: RequestInit) => {
      calls += 1
      expect(String(init.body)).not.toContain("me {")
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", f)
    expect(calls).toBe(1)
  })
})

describe("cíl zvonku", () => {
  async function bellInput(target?: { link?: string; action?: string }): Promise<Record<string, unknown>> {
    writeSecret("t")
    let gqlBody = ""
    const f = (async (u: unknown, init: RequestInit) => {
      if (String(u).endsWith("/graphql")) gqlBody = String(init.body)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("Předmět", f, target)
    return JSON.parse(gqlBody).variables.input
  }

  it("předá odkaz i popisek akce do createNotification", async () => {
    // Without this the bell is a dead end: a count with nothing to click.
    const input = await bellInput({ link: "https://sid.test/confirmations", action: "Otevřít v SID" })
    expect(input.link).toBe("https://sid.test/confirmations")
    expect(input.action).toBe("Otevřít v SID")
  })

  it("bez cíle pole vůbec neposílá (ne jako null)", async () => {
    const input = await bellInput()
    expect(input).not.toHaveProperty("link")
    expect(input).not.toHaveProperty("action")
  })
})

describe("daemon token - žádná header injection", () => {
  it("token s vnitřním newlinem se očistí, do Authorization se nedostane konec řádku", async () => {
    writeSecret("Bearer abc\nX-Injected: 1")
    let auth = ""
    const f = (async (_u: unknown, init: RequestInit) => {
      auth = new Headers(init.headers).get("authorization") ?? ""
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", f)
    expect(auth).not.toContain("\n")
    expect(auth).not.toContain("\r")
    expect(auth.startsWith("Bearer ")).toBe(true)
  })
})

describe("timeouty na volání daemona", () => {
  it("bell nese AbortSignal (zaseknutý daemon nesmí zablokovat cron)", async () => {
    writeSecret("t")
    const signals: unknown[] = []
    const f = (async (_u: unknown, init: RequestInit) => {
      signals.push(init.signal)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", f)
    expect(signals.length).toBeGreaterThan(0)
    for (const s of signals) expect(s).toBeInstanceOf(AbortSignal)
  })

  it("timeout daemona shodí jen notifikaci, ne celý běh", async () => {
    writeSecret("t")
    const f = (async () => {
      const e = new Error("aborted")
      e.name = "TimeoutError"
      throw e
    }) as unknown as typeof fetch
    // sendDigest is best-effort: it must return false, not throw.
    expect(await sendDigest("s", f)).toBe(false)
  })
})
