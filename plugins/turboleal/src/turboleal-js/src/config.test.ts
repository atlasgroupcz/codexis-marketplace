import { afterEach, describe, expect, it } from "bun:test"
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { apiKey, apiUrl, DEFAULT_API_URL, envPath } from "./config"
import { czechDate, sectionKey } from "./format"

const ENV = ["TURBOLEAL_API_KEY", "CODEXIS_PLUGIN_TURBOLEAL_API_URL", "CODEXIS_PUBLIC_USER_HOME"]
const saved = Object.fromEntries(ENV.map((name) => [name, process.env[name]]))

afterEach(() => {
  for (const name of ENV) {
    if (saved[name] === undefined) delete process.env[name]
    else process.env[name] = saved[name]
  }
})

describe("config", () => {
  it("reads the key from the app env file the daemon writes", () => {
    delete process.env.TURBOLEAL_API_KEY
    const home = mkdtempSync(join(tmpdir(), "turboleal-home-"))
    process.env.CODEXIS_PUBLIC_USER_HOME = home
    mkdirSync(join(home, ".cdx", "apps", "turboleal"), { recursive: true })
    writeFileSync(envPath(), "# key\n TURBOLEAL_API_KEY = api-key-stored \nOTHER=1\n")

    expect(envPath()).toBe(join(home, ".cdx", "apps", "turboleal", ".env"))
    expect(apiKey()).toBe("api-key-stored")

    process.env.TURBOLEAL_API_KEY = " api-key-env "
    expect(apiKey()).toBe("api-key-env")
  })

  it("uses the plugin default URL unless the env overrides it", () => {
    process.env.CODEXIS_PLUGIN_TURBOLEAL_API_URL = "  "
    expect(apiUrl()).toBe(DEFAULT_API_URL)

    process.env.CODEXIS_PLUGIN_TURBOLEAL_API_URL = "https://turboleal.example/"
    expect(apiUrl()).toBe("https://turboleal.example")
  })
})

describe("format", () => {
  it("writes Czech dates and reduces section labels", () => {
    expect(czechDate("2027-03-09")).toBe("9. 3. 2027")
    expect(czechDate("2026-10-15T12:00:00Z")).toBe("15. 10. 2026")
    expect(czechDate("soon")).toBe("soon")
    expect(sectionKey("§ 38g")).toBe(sectionKey("38g"))
    expect(sectionKey("§ 38ga")).not.toBe(sectionKey("38g"))
    expect(sectionKey("§ 72 - Nárok na odpočet daně")).toBe(sectionKey("72"))
  })
})
