import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { sendDigest, restBase, graphqlUrl } from "./notify"

let home: string
beforeEach(() => {
  home = mkdtempSync(join(tmpdir(), "sid-home-"))
  process.env.CODEXIS_PUBLIC_USER_HOME = home
  // CODEXIS_PUBLIC_DAEMON_URL is the GraphQL URL; the REST base derives from it.
  process.env.CODEXIS_PUBLIC_DAEMON_URL = "http://daemon.local/graphql"
  // E-mail is OFF by default (matching the peers), so tests of the e-mail
  // path have to enable it through the shipped defaults.
  const defaults = join(home, "defaults.json")
  writeFileSync(defaults, JSON.stringify({ notify: { email: true, inApp: true } }))
  process.env.SID_DEFAULTS_FILE = defaults
})

function writeSecret(value: string): void {
  mkdirSync(join(home, ".cdx", "env"), { recursive: true })
  writeFileSync(join(home, ".cdx", "env", "secret"), value)
}

describe("URL odvození", () => {
  it("restBase odřízne /graphql", () => {
    expect(restBase("http://d/graphql")).toBe("http://d")
    expect(restBase("http://d/graphql/")).toBe("http://d")
    expect(restBase("http://d")).toBe("http://d")
  })

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
    expect(await sendDigest("s", "b", f)).toBe(false)
    expect(called).toBe(false)
  })

  it("posílá multipart na /rest/v1/plugin/email/send s Bearer tokenem", async () => {
    writeSecret("vmtoken\n")
    const seen: { url?: string; auth?: string; ctype?: string; body?: string } = {}
    const f = (async (u: unknown, init: RequestInit) => {
      const h = new Headers(init.headers)
      // the bell (GraphQL) is called too; only the e-mail matters here
      if (String(u).includes("/rest/")) {
        seen.url = String(u)
        seen.auth = h.get("authorization") ?? ""
        seen.ctype = h.get("content-type") ?? ""
        seen.body = String(init.body)
      }
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch

    expect(await sendDigest("Předmět", "Tělo", f)).toBe(true)
    expect(seen.url).toBe("http://daemon.local/rest/v1/plugin/email/send")
    expect(seen.auth).toBe("Bearer vmtoken")
    expect(seen.ctype).toContain("multipart/form-data; boundary=")
    // the payload part carries JSON with subject + bodyText
    expect(seen.body).toContain('name="payload"')
    expect(seen.body).toContain('"subject":"Předmět"')
    expect(seen.body).toContain('"bodyText":"Tělo"')
  })

  it("normalizuje 'Authorization:' i holý token na Bearer", async () => {
    writeSecret("Authorization: Bearer abc\n")
    let auth = ""
    const f = (async (u: unknown, init: RequestInit) => {
      if (String(u).includes("/rest/")) auth = new Headers(init.headers).get("authorization") ?? ""
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", "b", f)
    expect(auth).toBe("Bearer abc")
  })

  it("posílá i in-app bell přes GraphQL createNotification", async () => {
    writeSecret("t")
    let gqlBody = ""
    const f = (async (u: unknown, init: RequestInit) => {
      if (String(u).endsWith("/graphql")) gqlBody = String(init.body)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("Předmět", "Tělo", f)
    expect(gqlBody).toContain("createNotification")
    expect(JSON.parse(gqlBody).variables.input.message).toBe("Předmět")
  })

  it("respektuje vypnuté kanály — vypnutý e-mail se neposílá", async () => {
    writeSecret("t")
    writeFileSync(process.env.SID_DEFAULTS_FILE!, JSON.stringify({ notify: { email: false, inApp: true } }))
    const hit = { rest: false, gql: false }
    const f = (async (u: unknown) => {
      if (String(u).includes("/rest/")) hit.rest = true
      if (String(u).endsWith("/graphql")) hit.gql = true
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    // True because the bell succeeded; keying the result to e-mail alone
    // would report failure on a working notification.
    expect(await sendDigest("s", "b", f)).toBe(true)
    expect(hit.rest).toBe(false)
    expect(hit.gql).toBe(true)
  })

  it("oba kanály vypnuté → nic se nevolá", async () => {
    writeSecret("t")
    writeFileSync(process.env.SID_DEFAULTS_FILE!, JSON.stringify({ notify: { email: false, inApp: false } }))
    let called = false
    const f = (async () => {
      called = true
      return new Response("{}")
    }) as unknown as typeof fetch
    expect(await sendDigest("s", "b", f)).toBe(false)
    expect(called).toBe(false)
  })

  it("HTTP chyba e-mailu → false, nikdy nehází", async () => {
    writeSecret("t")
    const f = (async () => new Response("x", { status: 500 })) as unknown as typeof fetch
    expect(await sendDigest("s", "b", f)).toBe(false)
  })

  it("výjimka transportu → false, nikdy nehází", async () => {
    writeSecret("t")
    const f = (async () => {
      throw new Error("ECONNREFUSED")
    }) as unknown as typeof fetch
    expect(await sendDigest("s", "b", f)).toBe(false)
  })
})

describe("recipients v e-mailu", () => {
  it("bez adresátů neposílá pole 'to' (daemon doručí přihlášenému)", async () => {
    writeSecret("t")
    let body = ""
    const f = (async (u: unknown, init: RequestInit) => {
      if (String(u).includes("/rest/")) body = String(init.body)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", "b", f)
    expect(body).not.toContain('"to"')
  })

  it("s adresáty je přiloží do payloadu", async () => {
    writeSecret("t")
    writeFileSync(
      process.env.SID_DEFAULTS_FILE!,
      JSON.stringify({ notify: { email: true, inApp: true, recipients: ["sef@firma.cz"] } }),
    )
    let body = ""
    const f = (async (u: unknown, init: RequestInit) => {
      if (String(u).includes("/rest/")) body = String(init.body)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", "b", f)
    expect(body).toContain('"to":["sef@firma.cz"]')
  })
})

describe("resolve_to — přihlášený uživatel nesmí vypadnout", () => {
  function withRecipients(list: string[]): void {
    writeFileSync(
      process.env.SID_DEFAULTS_FILE!,
      JSON.stringify({ notify: { email: true, inApp: false, recipients: list } }),
    )
  }

  it("k adresátům navíc přidá adresu přihlášeného uživatele", async () => {
    writeSecret("t")
    withRecipients(["sef@firma.cz"])
    let body = ""
    const f = (async (u: unknown, init: RequestInit) => {
      if (String(u).endsWith("/graphql")) {
        return new Response(JSON.stringify({ data: { me: { email: "ja@firma.cz" } } }), { status: 200 })
      }
      body = String(init.body)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", "b", f)
    // Without this, adding a colleague would switch off the user's own mail:
    // the daemon delivers EXCLUSIVELY to `to` when it is present.
    expect(JSON.parse(body.split("\r\n\r\n")[1]!.split("\r\n")[0]!).to).toEqual([
      "ja@firma.cz",
      "sef@firma.cz",
    ])
  })

  it("nezdvojí adresu, když si ji uživatel přidal sám", async () => {
    writeSecret("t")
    withRecipients(["JA@firma.cz", "sef@firma.cz"])
    let body = ""
    const f = (async (u: unknown, init: RequestInit) => {
      if (String(u).endsWith("/graphql")) {
        return new Response(JSON.stringify({ data: { me: { email: "ja@firma.cz" } } }), { status: 200 })
      }
      body = String(init.body)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", "b", f)
    expect(JSON.parse(body.split("\r\n\r\n")[1]!.split("\r\n")[0]!).to).toEqual([
      "ja@firma.cz",
      "sef@firma.cz",
    ])
  })

  it("když se `me` nepodaří zjistit, pošle aspoň extras (nezahodí je)", async () => {
    writeSecret("t")
    withRecipients(["sef@firma.cz"])
    let body = ""
    const f = (async (u: unknown, init: RequestInit) => {
      if (String(u).endsWith("/graphql")) return new Response("{}", { status: 500 })
      body = String(init.body)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", "b", f)
    expect(JSON.parse(body.split("\r\n\r\n")[1]!.split("\r\n")[0]!).to).toEqual(["sef@firma.cz"])
  })
})

describe("timeouty na volání daemona", () => {
  it("e-mail i bell nesou AbortSignal (zaseknutý daemon nesmí zablokovat cron)", async () => {
    writeSecret("t")
    const signals: unknown[] = []
    const f = (async (_u: unknown, init: RequestInit) => {
      signals.push(init.signal)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", "b", f)
    expect(signals.length).toBeGreaterThan(0)
    for (const s of signals) expect(s).toBeInstanceOf(AbortSignal)
  })

  it("dotaz na `me` nese AbortSignal taky", async () => {
    writeSecret("t")
    writeFileSync(
      process.env.SID_DEFAULTS_FILE!,
      JSON.stringify({ notify: { email: true, inApp: false, recipients: ["sef@firma.cz"] } }),
    )
    let meSignal: unknown = "nevolano"
    const f = (async (u: unknown, init: RequestInit) => {
      if (String(u).endsWith("/graphql")) meSignal = init.signal
      return new Response(JSON.stringify({ data: { me: { email: "ja@firma.cz" } } }), { status: 200 })
    }) as unknown as typeof fetch
    await sendDigest("s", "b", f)
    expect(meSignal).toBeInstanceOf(AbortSignal)
  })

  it("timeout daemona shodí jen notifikaci, ne celý běh", async () => {
    writeSecret("t")
    const f = (async () => {
      const e = new Error("aborted")
      e.name = "TimeoutError"
      throw e
    }) as unknown as typeof fetch
    // sendDigest is best-effort: it must return false, not throw.
    expect(await sendDigest("s", "b", f)).toBe(false)
  })
})
