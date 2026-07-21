import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./settings"
import { getApiKey } from "../config"
import type { CliError } from "../http"

beforeEach(() => {
  process.env.SID_ENV_FILE = join(mkdtempSync(join(tmpdir(), "sid-set-")), ".env")
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

const okUsersMe = (async () =>
  new Response(JSON.stringify({ id: "u1", email: "a@b.cz" }), { status: 200 })) as unknown as typeof fetch

describe("settings", () => {
  it("set-key čte klíč ze STDIN (nikdy z argv) a ověří ho", async () => {
    const out = (await run(["set-key"], okUsersMe, async () => "  sid_novy_klic\n")) as {
      verified: boolean
      start: string
    }
    expect(getApiKey()).toBe("sid_novy_klic")
    expect(out.verified).toBe(true)
    // the output must carry only the prefix, never the whole key
    expect(JSON.stringify(out)).not.toContain("sid_novy_klic")
    expect(out.start).toBe("sid_nov")
  })

  it("set-key s prázdným stdinem → USAGE", async () => {
    const err = (await run(["set-key"], okUsersMe, async () => "").catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
  })

  it("set-key odmítne klíč s newlinem (žádná .env injection), klíč se neuloží", async () => {
    let called = false
    const f = (async () => {
      called = true
      return new Response("{}")
    }) as unknown as typeof fetch
    const err = (await run(["set-key"], f, async () => "sid_a\nSID_API_URL=http://evil").catch(
      (e: unknown) => e,
    )) as CliError
    expect(err.exitCode).toBe(2)
    expect(getApiKey()).toBe("")
    expect(called).toBe(false) // rejected before even verifying against SID
  })

  it("po delete-key status nehlásí starou roli/isAdmin", async () => {
    const adminMe = (async () =>
      new Response(JSON.stringify({ id: "u1", email: "a@b.cz", role: "ADMIN" }), {
        status: 200,
      })) as unknown as typeof fetch
    await run(["set-key"], adminMe, async () => "sid_admin_key")
    await run(["delete-key"])
    const out = (await run(["status"])) as Record<string, unknown>
    expect(out.configured).toBe(false)
    expect(out.role).toBeUndefined()
    expect(out.isAdmin).toBeUndefined()
  })

  it("set-key s neplatným klíčem klíč NEuloží (rollback)", async () => {
    const reject = (async () => new Response("nope", { status: 401 })) as unknown as typeof fetch
    const err = (await run(["set-key"], reject, async () => "sid_spatny").catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(3)
    expect(getApiKey()).toBe("")
  })

  it("test bez klíče → AUTH s návodem", async () => {
    const err = (await run(["test"], okUsersMe).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(3)
  })

  it("status vrací jen configured/baseUrl/start", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_abcdef123456\n", { mode: 0o600 })
    const out = await run(["status"])
    expect(out).toEqual({ configured: true, baseUrl: "https://sid.onprem.agrp.dev", start: "sid_abc" })
  })

  it("status bez klíče nemá start", async () => {
    const out = await run(["status"])
    expect(out).toEqual({ configured: false, baseUrl: "https://sid.onprem.agrp.dev" })
  })

  it("delete-key smaže klíč", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_x\n", { mode: 0o600 })
    await run(["delete-key"])
    expect(getApiKey()).toBe("")
  })

  it("neznámý podpříkaz → USAGE", async () => {
    const err = (await run(["wat"]).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
  })
})

describe("set-key — neničí funkční klíč", () => {
  it("při nedostupném SID ponechá původní klíč a nehlásí 'odmítl'", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_puvodni\n", { mode: 0o600 })
    const down = (async () => {
      throw new Error("ECONNREFUSED")
    }) as unknown as typeof fetch
    const err = (await run(["set-key"], down, async () => "sid_novy").catch((e: unknown) => e)) as CliError
    // A working key must survive a network outage.
    expect(getApiKey()).toBe("sid_puvodni")
    // The message must not claim rejection when SID was simply unreachable.
    expect(err.message).not.toMatch(/odmítl/i)
  })

  it("při 401 původní klíč taky zůstane (nový se neuloží)", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_puvodni\n", { mode: 0o600 })
    const reject = (async () => new Response("nope", { status: 401 })) as unknown as typeof fetch
    await run(["set-key"], reject, async () => "sid_spatny").catch(() => undefined)
    expect(getApiKey()).toBe("sid_puvodni")
  })
})

describe("settings whoami", () => {
  it("reports who the key belongs to and whether they are an admin", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_x\n", { mode: 0o600 })
    const me = (async () =>
      new Response(JSON.stringify({ id: "u1", email: "sef@firma.cz", name: "Šéf", role: "ADMIN" }), {
        status: 200,
      })) as unknown as typeof fetch
    const out = (await run(["whoami"], me)) as {
      userId: string
      email: string
      role: string
      isAdmin: boolean
    }
    expect(out).toMatchObject({ userId: "u1", email: "sef@firma.cz", role: "ADMIN", isAdmin: true })
  })

  it("a regular user is not flagged as admin", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_x\n", { mode: 0o600 })
    const me = (async () =>
      new Response(JSON.stringify({ id: "u2", email: "a@b.cz", role: "USER" }), {
        status: 200,
      })) as unknown as typeof fetch
    const out = (await run(["whoami"], me)) as { role: string; isAdmin: boolean }
    expect(out.role).toBe("USER")
    expect(out.isAdmin).toBe(false)
  })

  it("an unknown role degrades to non-admin, never the other way", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_x\n", { mode: 0o600 })
    const me = (async () =>
      new Response(JSON.stringify({ id: "u3", email: "a@b.cz" }), { status: 200 })) as unknown as typeof fetch
    const out = (await run(["whoami"], me)) as { role: string; isAdmin: boolean }
    expect(out.isAdmin).toBe(false)
  })
})

describe("settings status — role", () => {
  it("surfaces the cached role so a caller can tell it is an admin", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_x\nSID_USER_ROLE=ADMIN\n", { mode: 0o600 })
    const out = (await run(["status"])) as { role?: string; isAdmin?: boolean }
    expect(out.role).toBe("ADMIN")
    expect(out.isAdmin).toBe(true)
  })

  it("omits the role when it has not been cached yet", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_x\n", { mode: 0o600 })
    const out = (await run(["status"])) as Record<string, unknown>
    expect(out).not.toHaveProperty("role")
  })
})
