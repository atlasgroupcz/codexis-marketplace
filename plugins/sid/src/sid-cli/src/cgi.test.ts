import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { handle, renderCgi } from "./cgi"
import { getApiKey } from "./config"

beforeEach(() => {
  process.env.SID_ENV_FILE = join(mkdtempSync(join(tmpdir(), "sid-cgi-")), ".env")
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

const okMe = (async () =>
  new Response(JSON.stringify({ id: "u1", email: "a@b.cz" }), { status: 200 })) as unknown as typeof fetch

describe("cgi handle", () => {
  it("save_config uloží klíč, ověří ho a NEvrátí ho zpět", async () => {
    const out = (await handle({ action: "save_config", api_key: "sid_klic123" }, okMe)) as {
      ok: boolean
      start?: string
    }
    expect(out.ok).toBe(true)
    expect(getApiKey()).toBe("sid_klic123")
    expect(JSON.stringify(out)).not.toContain("sid_klic123")
    expect(out.start).toBe("sid_kli")
  })

  it("save_config s neplatným klíčem klíč NEuloží", async () => {
    const reject = (async () => new Response("nope", { status: 401 })) as unknown as typeof fetch
    const out = (await handle({ action: "save_config", api_key: "sid_spatny" }, reject)) as { ok: boolean }
    expect(out.ok).toBe(false)
    expect(getApiKey()).toBe("")
  })

  it("save_config bez klíče → ok:false", async () => {
    const out = (await handle({ action: "save_config", api_key: "  " }, okMe)) as { ok: boolean }
    expect(out.ok).toBe(false)
  })

  it("get_status hlásí configured bez klíče", async () => {
    const out = await handle({ action: "get_status" }, okMe)
    expect(out).toMatchObject({ ok: true, configured: false })
  })

  it("delete_config smaže klíč", async () => {
    await handle({ action: "save_config", api_key: "sid_x" }, okMe)
    await handle({ action: "delete_config" }, okMe)
    expect(getApiKey()).toBe("")
  })

  it("neznámá akce → ok:false", async () => {
    const out = (await handle({ action: "wat" }, okMe)) as { ok: boolean }
    expect(out.ok).toBe(false)
  })
})

describe("renderCgi", () => {
  it("emituje CGI hlavičky oddělené prázdným řádkem", () => {
    const out = renderCgi({ ok: true })
    expect(out).toContain("Content-Type: application/json; charset=utf-8\r\n")
    expect(out).toContain("Cache-Control: no-store\r\n")
    const [head, body] = out.split("\r\n\r\n")
    expect(head).not.toContain("{")
    expect(JSON.parse(body!)).toEqual({ ok: true })
  })

  it("chybová odpověď nese Status hlavičku", () => {
    const out = renderCgi({ ok: false, error: "x" }, "400 Bad Request")
    expect(out.startsWith("Status: 400 Bad Request\r\n")).toBe(true)
  })
})
