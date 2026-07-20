import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { sendDigest, restBase, graphqlUrl } from "./notify"

let home: string
beforeEach(() => {
  home = mkdtempSync(join(tmpdir(), "sid-home-"))
  process.env.CODEXIS_PUBLIC_USER_HOME = home
  // CODEXIS_PUBLIC_DAEMON_URL je GraphQL URL — REST base se z něj odvozuje.
  process.env.CODEXIS_PUBLIC_DAEMON_URL = "http://daemon.local/graphql"
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
      // bell (GraphQL) voláme taky — zajímá nás jen e-mail
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
    // payload část nese JSON se subject + bodyText
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
