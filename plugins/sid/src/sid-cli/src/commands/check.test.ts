import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./check"

beforeEach(() => {
  const dir = mkdtempSync(join(tmpdir(), "sid-check-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  // No ~/.cdx/env/secret → notify is a no-op, notified stays false.
  process.env.CODEXIS_PUBLIC_USER_HOME = dir
  process.env.CODEXIS_PUBLIC_DAEMON_URL = "http://daemon.local/graphql"
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=sid_t\nSID_API_URL=http://s\nSID_USER_ROLE=USER\n", { mode: 0o600 })
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

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
