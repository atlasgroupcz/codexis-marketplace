import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./notifications"

beforeEach(() => {
  const dir = mkdtempSync(join(tmpdir(), "sid-notif-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=sid_t\nSID_API_URL=http://s\n", { mode: 0o600 })
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

function capturingFetch(cap: { url?: string }): typeof fetch {
  return (async (u: unknown) => {
    cap.url = String(u)
    return new Response("[]", { status: 200 })
  }) as unknown as typeof fetch
}

describe("notifications", () => {
  it("default = jen nepřečtené s limitem 50", async () => {
    const cap: { url?: string } = {}
    await run([], capturingFetch(cap))
    expect(cap.url).toBe("http://s/api/notifications?unreadOnly=true&limit=50")
  })

  it("--all vypne unreadOnly", async () => {
    const cap: { url?: string } = {}
    await run(["--all", "--limit", "10"], capturingFetch(cap))
    expect(cap.url).toBe("http://s/api/notifications?limit=10")
  })
})
