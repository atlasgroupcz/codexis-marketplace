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
    // výstup nesmí obsahovat celý klíč — jen prefix
    expect(JSON.stringify(out)).not.toContain("sid_novy_klic")
    expect(out.start).toBe("sid_nov")
  })

  it("set-key s prázdným stdinem → USAGE", async () => {
    const err = (await run(["set-key"], okUsersMe, async () => "").catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
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
    expect(out).toEqual({ configured: true, baseUrl: "https://sid.codexis.cz", start: "sid_abc" })
  })

  it("status bez klíče nemá start", async () => {
    const out = await run(["status"])
    expect(out).toEqual({ configured: false, baseUrl: "https://sid.codexis.cz" })
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
    // Původní funkční klíč musí přežít výpadek sítě.
    expect(getApiKey()).toBe("sid_puvodni")
    // Hláška nesmí tvrdit, že klíč byl odmítnut, když SID nebyl dostupný.
    expect(err.message).not.toMatch(/odmítl/i)
  })

  it("při 401 původní klíč taky zůstane (nový se neuloží)", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_puvodni\n", { mode: 0o600 })
    const reject = (async () => new Response("nope", { status: 401 })) as unknown as typeof fetch
    await run(["set-key"], reject, async () => "sid_spatny").catch(() => undefined)
    expect(getApiKey()).toBe("sid_puvodni")
  })
})
