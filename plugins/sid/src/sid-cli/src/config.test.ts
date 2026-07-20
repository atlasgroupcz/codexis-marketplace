import { describe, it, expect, beforeEach, afterEach } from "bun:test"
import { mkdtempSync, rmSync, statSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { envFilePath, readEnvFile, getApiKey, getBaseUrl, writeApiKey, deleteApiKey, EXIT } from "./config"

let dir: string
beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), "sid-cli-test-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})
afterEach(() => rmSync(dir, { recursive: true, force: true }))

describe("config", () => {
  it("EXIT kódy jsou stabilní kontrakt", () => {
    expect(EXIT).toEqual({ OK: 0, ERROR: 1, USAGE: 2, AUTH: 3, NOT_FOUND: 4, RATE_LIMIT: 5 })
  })

  it("bez souboru: prázdný klíč, default baseUrl", () => {
    expect(getApiKey()).toBe("")
    expect(getBaseUrl()).toBe("https://sid.onprem.agrp.dev")
  })

  it("writeApiKey zapíše 0600 a getApiKey ho přečte", () => {
    writeApiKey("sid_tajny123")
    expect(getApiKey()).toBe("sid_tajny123")
    const mode = statSync(envFilePath()).mode & 0o777
    expect(mode).toBe(0o600)
  })

  it("writeApiKey zachová SID_API_URL, deleteApiKey smaže jen klíč", () => {
    writeFileSync(envFilePath(), "SID_API_URL=http://localhost:3000\n")
    writeApiKey("sid_abc")
    expect(readEnvFile().SID_API_URL).toBe("http://localhost:3000")
    expect(getBaseUrl()).toBe("http://localhost:3000")
    deleteApiKey()
    expect(getApiKey()).toBe("")
    expect(getBaseUrl()).toBe("http://localhost:3000")
  })

  it("env proměnná CODEXIS_PLUGIN_SID_API_URL má přednost před .env", () => {
    writeFileSync(envFilePath(), "SID_API_URL=http://z-env-souboru\n")
    process.env.CODEXIS_PLUGIN_SID_API_URL = "https://z-daemona"
    expect(getBaseUrl()).toBe("https://z-daemona")
  })
})

describe("getBaseUrl — normalizace", () => {
  it("odřízne koncové lomítko (jinak vznikne //api/... na drátě)", () => {
    process.env.CODEXIS_PLUGIN_SID_API_URL = "https://sid.onprem.agrp.dev/"
    expect(getBaseUrl()).toBe("https://sid.onprem.agrp.dev")
  })

  it("odřízne i víc lomítek a bílé znaky", () => {
    process.env.CODEXIS_PLUGIN_SID_API_URL = "  https://sid.onprem.agrp.dev//  "
    expect(getBaseUrl()).toBe("https://sid.onprem.agrp.dev")
  })
})
