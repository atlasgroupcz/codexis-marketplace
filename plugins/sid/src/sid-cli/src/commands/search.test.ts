import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./search"
import type { CliError } from "../http"

beforeEach(() => {
  const dir = mkdtempSync(join(tmpdir(), "sid-search-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=sid_t\nSID_API_URL=http://s\nSID_USER_ROLE=USER\n", { mode: 0o600 })
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

function fetchReturning(payload: unknown, capture: { url?: string } = {}): typeof fetch {
  return (async (url: unknown) => {
    capture.url = String(url)
    return new Response(JSON.stringify(payload), { status: 200 })
  }) as unknown as typeof fetch
}

describe("search", () => {
  it("volá /api/search s q a size a vrací hits", async () => {
    const cap: { url?: string } = {}
    const out = (await run(
      ["smlouva o dílo", "--size", "5"],
      fetchReturning({ total: 2, hits: [{ documentId: "d1" }], source: "opensearch" }, cap),
    )) as { hits: unknown[]; warning?: string }
    expect(cap.url).toBe("http://s/api/search?q=smlouva+o+d%C3%ADlo&size=5")
    expect(out.hits).toHaveLength(1)
    // Every hit must carry the link into SID — the plugin leads users to the
    // app, it does not stand in for it.
    expect((out.hits[0] as { url: string }).url).toBe("http://s/files/d1")
    expect(out.warning).toBeUndefined()
  })

  it("opensearch_unavailable → warning pole (200 ≠ nic nenalezeno)", async () => {
    const out = (await run(["x"], fetchReturning({ total: 0, hits: [], source: "opensearch_unavailable" }))) as {
      warning?: string
    }
    expect(out.warning).toMatch(/nedostupn/i)
  })

  it("bez dotazu → USAGE", async () => {
    const err = (await run([]).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
  })
})
