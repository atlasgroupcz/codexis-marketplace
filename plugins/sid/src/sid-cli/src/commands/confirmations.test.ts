import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./confirmations"
import type { CliError } from "../http"

beforeEach(() => {
  const dir = mkdtempSync(join(tmpdir(), "sid-conf-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=sid_t\nSID_API_URL=http://s\nSID_USER_ROLE=USER\n", { mode: 0o600 })
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

describe("confirmations", () => {
  it("list --state pending → GET s query", async () => {
    const cap: { url?: string } = {}
    const f = (async (u: unknown) => {
      cap.url = String(u)
      return new Response("[]", { status: 200 })
    }) as unknown as typeof fetch
    await run(["list", "--state", "pending"], f)
    expect(cap.url).toBe("http://s/api/v1/confirmations?state=pending")
  })

  it("list bez state → bez query stringu", async () => {
    const cap: { url?: string } = {}
    const f = (async (u: unknown) => {
      cap.url = String(u)
      return new Response("[]", { status: 200 })
    }) as unknown as typeof fetch
    await run(["list"], f)
    expect(cap.url).toBe("http://s/api/v1/confirmations")
  })

  it("each pending item carries the SID link and the app URL", async () => {
    const f = (async () =>
      new Response(JSON.stringify([{ id: "c1", documentId: "d1" }]), { status: 200 })) as unknown as typeof fetch
    const out = (await run(["list", "--state", "pending"], f)) as {
      confirmations: { url: string }[]
      confirmInApp: string
    }
    expect(out.confirmations[0]!.url).toBe("http://s/files/d1")
    // Confirming is a legal act and belongs in SID, so the CLI points there.
    expect(out.confirmInApp).toBe("http://s/confirmations")
  })

  it("confirm is no longer a CLI command — it must happen in SID", async () => {
    const err = (await run(["confirm", "c1"]).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
    expect(err.message).toMatch(/SID/)
  })

  it("neplatný --state → USAGE, server se nevolá", async () => {
    let called = false
    const f = (async () => {
      called = true
      return new Response("[]")
    }) as unknown as typeof fetch
    const err = (await run(["list", "--state", "bogus"], f).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
    expect(called).toBe(false)
  })
})
