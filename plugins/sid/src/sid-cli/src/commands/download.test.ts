import { describe, it, expect, beforeEach, afterEach } from "bun:test"
import { mkdtempSync, rmSync, writeFileSync, readFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./download"
import type { CliError } from "../http"

let dir: string
beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), "sid-dl-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=sid_t\nSID_API_URL=http://s\n", { mode: 0o600 })
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})
afterEach(() => rmSync(dir, { recursive: true, force: true }))

describe("download", () => {
  it("uloží tělo do --out a vrátí savedTo+bytes", async () => {
    const f = (async () =>
      new Response("PDFDATA", {
        status: 200,
        headers: { "content-disposition": 'attachment; filename="smernice.pdf"' },
      })) as unknown as typeof fetch
    const out = join(dir, "x.pdf")
    const res = (await run(["file-1", "--out", out], f)) as { savedTo: string; bytes: number }
    expect(res.savedTo).toBe(out)
    expect(readFileSync(out, "utf8")).toBe("PDFDATA")
    expect(res.bytes).toBe(7)
  })

  it("bez --out odvodí jméno z content-disposition", async () => {
    const f = (async () =>
      new Response("D", {
        status: 200,
        headers: { "content-disposition": 'attachment; filename="rad.pdf"' },
      })) as unknown as typeof fetch
    const prev = process.cwd()
    process.chdir(dir)
    try {
      const res = (await run(["file-2"], f)) as { savedTo: string }
      expect(res.savedTo.endsWith("rad.pdf")).toBe(true)
    } finally {
      process.chdir(prev)
    }
  })

  it("bez fileId → USAGE", async () => {
    const err = (await run([]).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
  })
})

describe("download — bezpečnost jména souboru", () => {
  it("server nemůže path traversalem zapsat mimo cílový adresář", async () => {
    // The server controls content-disposition; without basename() ../ escapes cwd.
    const f = (async () =>
      new Response("X", {
        status: 200,
        headers: { "content-disposition": 'attachment; filename="../../../evil.sh"' },
      })) as unknown as typeof fetch
    const prev = process.cwd()
    process.chdir(dir)
    try {
      const res = (await run(["file-x"], f)) as { savedTo: string }
      expect(res.savedTo).toBe(join(dir, "evil.sh"))
      expect(res.savedTo).not.toContain("..")
    } finally {
      process.chdir(prev)
    }
  })

  it("absolutní cestu ze serveru zahodí a vezme jen jméno", async () => {
    const f = (async () =>
      new Response("X", {
        status: 200,
        headers: { "content-disposition": 'attachment; filename="/etc/cron.d/pwn"' },
      })) as unknown as typeof fetch
    const prev = process.cwd()
    process.chdir(dir)
    try {
      const res = (await run(["file-y"], f)) as { savedTo: string }
      expect(res.savedTo).toBe(join(dir, "pwn"))
    } finally {
      process.chdir(prev)
    }
  })

  it("--out od uživatele zůstává respektovaný (i mimo cwd)", async () => {
    const f = (async () => new Response("X", { status: 200 })) as unknown as typeof fetch
    const out = join(dir, "sub", "moje.pdf")
    const res = (await run(["file-z", "--out", out], f)) as { savedTo: string }
    expect(res.savedTo).toBe(out)
  })
})
