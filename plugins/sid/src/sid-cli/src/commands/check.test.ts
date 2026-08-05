import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./check"

let home: string
beforeEach(() => {
  home = mkdtempSync(join(tmpdir(), "sid-check-"))
  process.env.SID_ENV_FILE = join(home, ".env")
  // No ~/.cdx/env/secret → notify is a no-op, notified stays false.
  process.env.CODEXIS_PUBLIC_USER_HOME = home
  process.env.CODEXIS_PUBLIC_DAEMON_URL = "http://daemon.local/graphql"
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=sid_t\nSID_API_URL=http://s\nSID_USER_ROLE=USER\n", { mode: 0o600 })
  // Pin the shipped defaults into the temp home: without this the channel
  // config would come from whatever the machine (or a sibling test file)
  // left behind, making the notify assertions order-dependent.
  const defaults = join(home, "defaults.json")
  writeFileSync(defaults, JSON.stringify({ notify: { inApp: true } }))
  process.env.SID_DEFAULTS_FILE = defaults
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

/** Arms the daemon channel - without the secret `sendDigest` is a no-op. */
function writeSecret(): void {
  mkdirSync(join(home, ".cdx", "env"), { recursive: true })
  writeFileSync(join(home, ".cdx", "env", "secret"), "vmtoken")
}

function fakeSid(): typeof fetch {
  return (async (u: unknown) => {
    const url = String(u)
    if (url.includes("/api/notifications")) {
      return new Response(JSON.stringify([{ id: "n1", title: "Nový dokument" }]))
    }
    if (url.includes("/api/v1/confirmations")) {
      return new Response(JSON.stringify([{ id: "c1" }, { id: "c2" }]))
    }
    return new Response("{}")
  }) as unknown as typeof fetch
}

describe("check", () => {
  it("sečte nepřečtené a čekající", async () => {
    const out = (await run([], fakeSid())) as {
      unreadNotifications: number
      pendingConfirmations: number
      notified: boolean
    }
    expect(out.unreadNotifications).toBe(1)
    expect(out.pendingConfirmations).toBe(2)
    expect(out.notified).toBe(false)
  })

  it("bez --notify neposílá nic ani když je co hlásit", async () => {
    let notifyCalled = false
    const f = (async (u: unknown) => {
      const url = String(u)
      if (url.includes("daemon.local")) notifyCalled = true
      if (url.includes("/api/notifications")) return new Response(JSON.stringify([{ id: "n1" }]))
      return new Response("[]")
    }) as unknown as typeof fetch
    await run([], f)
    expect(notifyCalled).toBe(false)
  })

  it("--notify bez ničeho nového neposílá nic", async () => {
    const empty = (async () => new Response("[]")) as unknown as typeof fetch
    const out = (await run(["--notify"], empty)) as { notified: boolean }
    expect(out.notified).toBe(false)
  })
})

describe("odkaz ve zvonku", () => {
  /** Runs `check --notify` against a SID with the given payloads and returns
   *  the `createNotification` input the daemon received. */
  async function bellInput(notifications: unknown[], confirmations: unknown[]): Promise<Record<string, unknown>> {
    writeSecret()
    let gqlBody = ""
    const f = (async (u: unknown, init: RequestInit) => {
      const url = String(u)
      if (url.includes("daemon.local")) {
        gqlBody = String(init.body)
        return new Response("{}", { status: 200 })
      }
      if (url.includes("/api/notifications")) return new Response(JSON.stringify(notifications))
      if (url.includes("/api/v1/confirmations")) return new Response(JSON.stringify(confirmations))
      return new Response("{}")
    }) as unknown as typeof fetch
    const out = (await run(["--notify"], f)) as { notified: boolean }
    expect(out.notified).toBe(true)
    return JSON.parse(gqlBody).variables.input
  }

  it("s čekajícím potvrzením vede rovnou na seznam potvrzení", async () => {
    const input = await bellInput([], [{ id: "c1" }])
    expect(input.link).toBe("http://s/confirmations")
    expect(input.action).toBe("Otevřít v SID")
  })

  it("bez čekajících potvrzení vede na SID, ne na prázdný seznam potvrzení", async () => {
    const input = await bellInput([{ id: "n1" }], [])
    expect(input.link).toBe("http://s")
  })

  it("hlásí počty ve zprávě - zvonek je jediný kanál, musí nést celou informaci", async () => {
    const input = await bellInput([{ id: "n1" }], [{ id: "c1" }, { id: "c2" }])
    expect(input.message).toBe("SID: 1 nových notifikací, 2 čekajících potvrzení")
  })

  it("odkaz respektuje roli - admin nesmí skončit na uživatelské routě", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=sid_t\nSID_API_URL=http://s\nSID_USER_ROLE=ADMIN\n", {
      mode: 0o600,
    })
    const input = await bellInput([], [{ id: "c1" }])
    expect(input.link).toBe("http://s/admin/confirmations")
  })
})
