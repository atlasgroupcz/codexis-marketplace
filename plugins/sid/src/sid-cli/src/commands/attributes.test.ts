import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./attributes"

beforeEach(() => {
  const dir = mkdtempSync(join(tmpdir(), "sid-attrs-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=k\nSID_API_URL=http://s\nSID_USER_ROLE=USER\n", {
    mode: 0o600,
  })
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

const ATTRS = [
  { id: "a1", name: "Platnost", type: "DATE", options: [], archivedAt: null, documentCount: 4 },
  { id: "a2", name: "Útvar", type: "STRING", options: ["Právní", "HR"], archivedAt: null, documentCount: 9 },
  { id: "a3", name: "Starý", type: "STRING", options: [], archivedAt: "2026-01-01", documentCount: 0 },
]

const f = (async () => new Response(JSON.stringify(ATTRS), { status: 200 })) as unknown as typeof fetch

describe("attributes list", () => {
  it("hides archived attributes by default — they cannot be assigned", async () => {
    const out = (await run(["list"], f)) as { attributes: Array<{ id: string }> }
    expect(out.attributes.map((a) => a.id)).toEqual(["a1", "a2"])
  })

  it("--all includes archived ones", async () => {
    const out = (await run(["list", "--all"], f)) as { attributes: Array<{ id: string }> }
    expect(out.attributes).toHaveLength(3)
  })

  it("keeps the fields needed to build a filter", async () => {
    const out = (await run(["list"], f)) as {
      attributes: Array<{ id: string; name: string; type: string; options: string[] }>
    }
    expect(out.attributes[1]).toMatchObject({
      id: "a2",
      name: "Útvar",
      type: "STRING",
      options: ["Právní", "HR"],
    })
  })
})
