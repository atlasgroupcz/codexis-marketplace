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
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=sid_t\nSID_API_URL=http://s\nSID_USER_ROLE=USER\n", { mode: 0o600 })
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

/**
 * `download` takes a fileId, but every listing and search result hands out
 * documentIds, and both are bare UUIDs. SID answers an unknown file id with
 * 403 (it won't confirm which ids exist), so the mix-up used to surface as
 * "Přístup odepřen" and the agent gave up on a document the user could read.
 *
 * So a documentId is now resolved to its current file and retried. The point
 * of these tests is that convenience buys no extra rights: BOTH calls go to
 * SID with the API key and SID gates both, so anything the user may not see
 * still fails.
 */
describe("download — documentId se doresí na fileId", () => {
  /** Router over (url → Response), recording every request in order. */
  function routed(handler: (url: string) => Response): { f: typeof fetch; calls: string[] } {
    const calls: string[] = []
    const f = (async (url: unknown) => {
      const u = String(url)
      calls.push(u)
      return handler(u)
    }) as unknown as typeof fetch
    return { f, calls }
  }

  const forbidden = () =>
    new Response(JSON.stringify({ error: { code: "FORBIDDEN", message: "Access denied" } }), {
      status: 403,
      headers: { "content-type": "application/json" },
    })

  it("fileId stáhne napřímo — žádné dohledávání navíc", async () => {
    const { f, calls } = routed(() => new Response("PDF", { status: 200 }))
    const out = join(dir, "a.pdf")

    await run(["file-1", "--out", out], f)

    expect(calls).toHaveLength(1)
    expect(calls[0]).toContain("/api/files/download/file-1")
  })

  it("documentId: po 403 si dojde pro currentFile.id a stáhne ho", async () => {
    const { f, calls } = routed((url) => {
      if (url.includes("/api/files/download/doc-1")) return forbidden()
      if (url.includes("/api/v1/documents/doc-1")) {
        return new Response(JSON.stringify({ id: "doc-1", currentFile: { id: "file-9" } }), {
          status: 200,
          headers: { "content-type": "application/json" },
        })
      }
      return new Response("PDFDATA", { status: 200 })
    })
    const out = join(dir, "b.pdf")

    const res = (await run(["doc-1", "--out", out], f)) as { savedTo: string; bytes: number }

    expect(readFileSync(out, "utf8")).toBe("PDFDATA")
    expect(res.savedTo).toBe(out)
    expect(calls[0]).toContain("/api/files/download/doc-1")
    expect(calls[1]).toContain("/api/v1/documents/doc-1")
    expect(calls[2]).toContain("/api/files/download/file-9")
  })

  // The one that matters: SID gates the lookup too, so a document the caller
  // may not read resolves to nothing and no bytes are ever fetched.
  it("na cizí dokument práva nedá — dohledání je taky hlídané a soubor se nestáhne", async () => {
    const { f, calls } = routed(() => forbidden())

    const err = (await run(["doc-cizi", "--out", join(dir, "c.pdf")], f).catch(
      (e: unknown) => e,
    )) as CliError

    expect(err.exitCode).toBe(6)
    expect(String(err.message)).toContain("403")
    // download(403) → docs get(403) → hlášení. Nikdy třetí, stahovací pokus.
    expect(calls).toHaveLength(2)
    expect(calls.some((u) => u.includes("/api/files/download/") && !u.includes("doc-cizi"))).toBe(false)
  })

  it("dokument bez souboru → srozumitelná chyba, ne prázdný soubor", async () => {
    const { f } = routed((url) => {
      if (url.includes("/api/v1/documents/")) {
        return new Response(JSON.stringify({ id: "doc-2", currentFile: null }), {
          status: 200,
          headers: { "content-type": "application/json" },
        })
      }
      return forbidden()
    })

    const err = (await run(["doc-2", "--out", join(dir, "d.pdf")], f).catch(
      (e: unknown) => e,
    )) as CliError

    expect(err.exitCode).toBe(6)
  })

  it("vypršelý klíč při dohledání se hlásí jako klíč, ne jako odepřený přístup", async () => {
    const { f } = routed((url) => {
      if (url.includes("/api/v1/documents/")) {
        return new Response(JSON.stringify({ error: { code: "UNAUTHORIZED" } }), {
          status: 401,
          headers: { "content-type": "application/json" },
        })
      }
      return forbidden()
    })

    const err = (await run(["doc-3", "--out", join(dir, "e.pdf")], f).catch(
      (e: unknown) => e,
    )) as CliError

    expect(err.exitCode).toBe(3)
  })

  it("dohledání proběhne nejvýš jednou — žádná smyčka", async () => {
    const { f, calls } = routed((url) => {
      if (url.includes("/api/v1/documents/")) {
        return new Response(JSON.stringify({ id: "doc-4", currentFile: { id: "file-x" } }), {
          status: 200,
          headers: { "content-type": "application/json" },
        })
      }
      return forbidden()
    })

    const err = (await run(["doc-4", "--out", join(dir, "f.pdf")], f).catch(
      (e: unknown) => e,
    )) as CliError

    expect(err.exitCode).toBe(6)
    expect(calls).toHaveLength(3)
  })
})
