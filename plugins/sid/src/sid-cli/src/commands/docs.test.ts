import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./docs"
import type { CliError } from "../http"

beforeEach(() => {
  const dir = mkdtempSync(join(tmpdir(), "sid-docs-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=sid_t\nSID_API_URL=http://s\nSID_USER_ROLE=USER\n", { mode: 0o600 })
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

const cap: { url?: string } = {}
const f = (async (url: unknown) => {
  cap.url = String(url)
  return new Response("{}", { status: 200 })
}) as unknown as typeof fetch

describe("docs", () => {
  it("list skládá filtry do query stringu", async () => {
    await run(["list", "--query", "řád", "--folder", "f1", "--page", "2"], f)
    expect(cap.url).toBe("http://s/api/v1/documents?query=%C5%99%C3%A1d&folderId=f1&page=2")
  })

  it("list bez filtrů nemá query string", async () => {
    await run(["list"], f)
    expect(cap.url).toBe("http://s/api/v1/documents")
  })

  it("get volá /api/v1/documents/[id]", async () => {
    await run(["get", "doc-123"], f)
    expect(cap.url).toBe("http://s/api/v1/documents/doc-123")
  })

  it("get bez id → USAGE", async () => {
    const err = (await run(["get"]).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
  })

  it("neznámý podpříkaz → USAGE", async () => {
    const err = (await run(["wat"]).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
  })
})

describe("docs list — the rest of the documented filters", () => {
  it("passes attributes through, repeatable (AND across attributes)", async () => {
    await run(["list", "--attribute", "a1", "--attribute", "a2"], f)
    expect(cap.url).toBe("http://s/api/v1/documents?attributeIds=a1&attributeIds=a2")
  })

  it("passes a date range", async () => {
    await run(["list", "--from", "2026-01-01", "--to", "2026-06-30"], f)
    expect(cap.url).toBe("http://s/api/v1/documents?dateFrom=2026-01-01&dateTo=2026-06-30")
  })

  it("passes the file type", async () => {
    await run(["list", "--type", "pdf"], f)
    expect(cap.url).toBe("http://s/api/v1/documents?mimeType=pdf")
  })

  it("--personal asks for documents with no group", async () => {
    await run(["list", "--personal"], f)
    expect(cap.url).toBe("http://s/api/v1/documents?personalOnly=true")
  })

  it("combines filters in a stable order", async () => {
    await run(["list", "--query", "řád", "--type", "pdf", "--attribute", "a1", "--personal"], f)
    expect(cap.url).toBe(
      "http://s/api/v1/documents?query=%C5%99%C3%A1d&mimeType=pdf&personalOnly=true&attributeIds=a1",
    )
  })
})
