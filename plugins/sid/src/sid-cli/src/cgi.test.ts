import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { handle, renderCgi } from "./cgi"
import { getApiKey } from "./config"

beforeEach(() => {
  const d = mkdtempSync(join(tmpdir(), "sid-cgi-"))
  process.env.SID_ENV_FILE = join(d, ".env")
  process.env.CODEXIS_PUBLIC_USER_HOME = d
  process.env.SID_DEFAULTS_FILE = join(d, "nic.json")
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

  it("save_config odmítne klíč s newlinem - žádná .env injection", async () => {
    const out = (await handle(
      { action: "save_config", api_key: "sid_a\nSID_API_URL=http://evil" },
      okMe,
    )) as { ok: boolean }
    expect(out.ok).toBe(false)
    expect(getApiKey()).toBe("")
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

describe("cgi - notifikace", () => {
  it("get_status vrací aktuální nastavení kanálů", async () => {
    const out = (await handle({ action: "get_status" }, okMe)) as { notify: unknown }
    expect(out.notify).toEqual({ email: false, inApp: true, recipients: [] })
  })

  it("save_notify_settings uloží přepínače", async () => {
    const out = (await handle({ action: "save_notify_settings", email: true }, okMe)) as {
      ok: boolean
      notify: { email: boolean }
    }
    expect(out.ok).toBe(true)
    expect(out.notify.email).toBe(true)
  })

  it("save_notify_settings odmítne neplatnou adresu a nic neuloží", async () => {
    const out = (await handle(
      { action: "save_notify_settings", recipients: ["tohle-neni-email"] },
      okMe,
    )) as { ok: boolean; error?: string }
    expect(out.ok).toBe(false)
    expect(out.error).toMatch(/Neplatná/i)
    const st = (await handle({ action: "get_status" }, okMe)) as { notify: { recipients: string[] } }
    expect(st.notify.recipients).toEqual([])
  })

  it("save_notify_settings přijme platné adresáty", async () => {
    const out = (await handle(
      { action: "save_notify_settings", recipients: ["sef@firma.cz"] },
      okMe,
    )) as { ok: boolean; notify: { recipients: string[] } }
    expect(out.ok).toBe(true)
    expect(out.notify.recipients).toEqual(["sef@firma.cz"])
  })
})
