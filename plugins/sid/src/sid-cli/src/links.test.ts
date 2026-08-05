import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync, readFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { documentUrl, homeUrl, resolveRole } from "./links"

beforeEach(() => {
  const d = mkdtempSync(join(tmpdir(), "sid-links-"))
  process.env.SID_ENV_FILE = join(d, ".env")
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=k\nSID_API_URL=https://sid.test\n", { mode: 0o600 })
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

describe("documentUrl", () => {
  it("regular user gets the /files route", () => {
    expect(documentUrl("doc-1", "USER")).toBe("https://sid.test/files/doc-1")
  })

  it("admin gets /admin/files — the (user) layout bounces admins to the dashboard", () => {
    expect(documentUrl("doc-1", "ADMIN")).toBe("https://sid.test/admin/files/doc-1")
  })

  it("unknown role falls back to the user route", () => {
    expect(documentUrl("doc-1", null)).toBe("https://sid.test/files/doc-1")
  })

  it("id is url-encoded", () => {
    expect(documentUrl("a b/c", "USER")).toBe("https://sid.test/files/a%20b%2Fc")
  })
})

describe("homeUrl", () => {
  it("is SID's front door — no trailing slash to double up on", () => {
    expect(homeUrl()).toBe("https://sid.test")
  })
})

describe("resolveRole", () => {
  it("uses the cached role without calling the API", async () => {
    writeFileSync(process.env.SID_ENV_FILE!, "SID_API_KEY=k\nSID_USER_ROLE=ADMIN\n", { mode: 0o600 })
    let called = false
    const f = (async () => {
      called = true
      return new Response("{}")
    }) as unknown as typeof fetch
    expect(await resolveRole(f)).toBe("ADMIN")
    expect(called).toBe(false)
  })

  it("fetches and caches the role when it is missing", async () => {
    const f = (async () =>
      new Response(JSON.stringify({ role: "ADMIN" }), { status: 200 })) as unknown as typeof fetch
    expect(await resolveRole(f)).toBe("ADMIN")
    expect(readFileSync(process.env.SID_ENV_FILE!, "utf8")).toContain("SID_USER_ROLE=ADMIN")
  })

  it("a failed lookup yields null instead of throwing", async () => {
    const f = (async () => {
      throw new Error("down")
    }) as unknown as typeof fetch
    expect(await resolveRole(f)).toBeNull()
  })
})
