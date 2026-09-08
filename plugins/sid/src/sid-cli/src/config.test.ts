import { describe, it, expect, beforeEach, afterEach } from "bun:test"
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { envFilePath, readEnvFile, getApiKey, getBaseUrl, writeApiKey, deleteApiKey, isPlausibleApiKey, EXIT } from "./config"
import { FakeCdxctl } from "./test-support"

let dir: string
let cdxctl: FakeCdxctl
beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), "sid-cli-test-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
  cdxctl = new FakeCdxctl(dir)
})
afterEach(() => {
  cdxctl.restore()
  rmSync(dir, { recursive: true, force: true })
})

describe("config", () => {
  it("EXIT kódy jsou stabilní kontrakt", () => {
    // Additive only: existing codes keep their meaning, new ones go on the end
    // and get a row in SKILL.md. FORBIDDEN (6) split off from AUTH (3) once it
    // turned out 403 was being reported to the model as an expired key.
    expect(EXIT).toEqual({
      OK: 0,
      ERROR: 1,
      USAGE: 2,
      AUTH: 3,
      NOT_FOUND: 4,
      RATE_LIMIT: 5,
      FORBIDDEN: 6,
    })
  })

  it("bez souboru: prázdný klíč, default baseUrl", () => {
    expect(getApiKey()).toBe("")
    expect(getBaseUrl()).toBe("https://sid.onprem.agrp.dev")
  })

  it("writeApiKey požádá platformu o uložení a sám .env nezapíše", () => {
    writeApiKey("sid_tajny123")
    expect(cdxctl.calls()).toEqual(["appenv sid set SID_API_KEY sid_tajny123"])
    expect(existsSync(envFilePath())).toBe(false)
  })

  it("getApiKey čte klíč ze souboru, který zapsal démon", () => {
    writeFileSync(envFilePath(), "SID_API_KEY=sid_tajny123\n")
    expect(getApiKey()).toBe("sid_tajny123")
  })

  it("deleteApiKey maže jen klíč, SID_API_URL nechá být", () => {
    writeFileSync(envFilePath(), "SID_API_KEY=sid_abc\nSID_API_URL=http://localhost:3000\n")

    deleteApiKey()

    expect(cdxctl.calls()).toEqual(["appenv sid delete SID_API_KEY"])
    expect(getBaseUrl()).toBe("http://localhost:3000")
  })

  it("selhání cdxctl vyplave jako chyba s kódem a stderr", () => {
    cdxctl.failWith(7)
    expect(() => writeApiKey("sid_abc")).toThrow(/7/)
  })

  it("env proměnná CODEXIS_PLUGIN_SID_API_URL má přednost před .env", () => {
    writeFileSync(envFilePath(), "SID_API_URL=http://z-env-souboru\n")
    process.env.CODEXIS_PLUGIN_SID_API_URL = "https://z-daemona"
    expect(getBaseUrl()).toBe("https://z-daemona")
  })

  it("isPlausibleApiKey: povolí base64url znaky, zakáže prázdný a bílé znaky", () => {
    expect(isPlausibleApiKey("sid_aB3-DE_f.1")).toBe(true)
    expect(isPlausibleApiKey("")).toBe(false)
    expect(isPlausibleApiKey("sid a")).toBe(false)
    // The injection vector: a newline would forge a second .env line.
    expect(isPlausibleApiKey("sid_a\nSID_API_URL=http://evil")).toBe(false)
  })

  it("writeApiKey odmítne klíč s newlinem dřív, než sáhne na platformu", () => {
    expect(() => writeApiKey("sid_a\nSID_API_URL=http://evil")).toThrow()
    expect(cdxctl.calls()).toEqual([])
  })

  it("deleteApiKey smaže i nacacheovanou roli a e-mail, ostatní nechá", () => {
    writeFileSync(
      envFilePath(),
      "SID_API_KEY=sid_a\nSID_USER_ROLE=ADMIN\nSID_USER_EMAIL=a@b.cz\nSID_API_URL=http://s\n",
    )

    deleteApiKey()

    expect(cdxctl.calls()).toEqual([
      "appenv sid delete SID_API_KEY",
      "appenv sid delete SID_USER_ROLE",
      "appenv sid delete SID_USER_EMAIL",
    ])
    expect(readEnvFile().SID_API_URL).toBe("http://s")
  })
})

describe("getBaseUrl - normalizace", () => {
  it("odřízne koncové lomítko (jinak vznikne //api/... na drátě)", () => {
    process.env.CODEXIS_PLUGIN_SID_API_URL = "https://sid.onprem.agrp.dev/"
    expect(getBaseUrl()).toBe("https://sid.onprem.agrp.dev")
  })

  it("odřízne i víc lomítek a bílé znaky", () => {
    process.env.CODEXIS_PLUGIN_SID_API_URL = "  https://sid.onprem.agrp.dev//  "
    expect(getBaseUrl()).toBe("https://sid.onprem.agrp.dev")
  })
})
