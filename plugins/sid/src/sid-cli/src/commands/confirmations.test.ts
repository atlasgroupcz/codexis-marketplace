import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./confirmations"
import type { CliError } from "../http"

beforeEach(() => {
  const dir = mkdtempSync(join(tmpdir(), "sid-conf-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=sid_t\nSID_API_URL=http://s\n", { mode: 0o600 })
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

  it("confirm <id> → PATCH {state: CONFIRMED}", async () => {
    let method = ""
    let body = ""
    const f = (async (_u: unknown, init: RequestInit) => {
      method = String(init.method)
      body = String(init.body)
      return new Response("{}", { status: 200 })
    }) as unknown as typeof fetch
    await run(["confirm", "c1"], f)
    expect(method).toBe("PATCH")
    expect(JSON.parse(body)).toEqual({ state: "CONFIRMED" })
  })

  it("confirm bez id → USAGE", async () => {
    const err = (await run(["confirm"]).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
  })
})
