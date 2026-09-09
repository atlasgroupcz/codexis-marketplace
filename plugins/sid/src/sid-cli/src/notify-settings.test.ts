import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, statSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { getNotifySettings } from "./notify-settings"

let home: string
let share: string
beforeEach(() => {
  home = mkdtempSync(join(tmpdir(), "sid-set-home-"))
  share = mkdtempSync(join(tmpdir(), "sid-set-share-"))
  process.env.CODEXIS_PUBLIC_USER_HOME = home
  process.env.SID_DEFAULTS_FILE = join(share, "defaults.json")
})

function writeDefaults(obj: unknown): void {
  writeFileSync(process.env.SID_DEFAULTS_FILE!, JSON.stringify(obj))
}

function writeUser(obj: unknown): void {
  const dir = join(home, ".cdx", "app-state", "sid")
  mkdirSync(dir, { recursive: true })
  writeFileSync(join(dir, "settings.json"), JSON.stringify(obj))
}

describe("getNotifySettings", () => {
  it("bez jakýchkoli souborů je bell zapnutý", () => {
    expect(getNotifySettings()).toEqual({ inApp: true })
  })

  it("čte shipped defaults", () => {
    writeDefaults({ version: 1, notify: { inApp: false } })
    expect(getNotifySettings()).toEqual({ inApp: false })
  })

  it("uživatelský override přebije defaults", () => {
    writeDefaults({ version: 1, notify: { inApp: true } })
    writeUser({ notify: { inApp: false } })
    expect(getNotifySettings()).toEqual({ inApp: false })
  })

  it("poškozený JSON nespadne, použije vestavěné hodnoty", () => {
    writeFileSync(process.env.SID_DEFAULTS_FILE!, "{ tohle není json")
    expect(getNotifySettings()).toEqual({ inApp: true })
  })

  it("nesmyslné typy ignoruje", () => {
    writeDefaults({ notify: { inApp: 1 } })
    expect(getNotifySettings()).toEqual({ inApp: true })
  })
})

describe("pozůstatky po e-mailovém kanálu", () => {
  // The plugin used to offer e-mail + extra recipients. SID sends its own
  // mail, so that channel is gone; a settings file written by the old
  // version must not resurrect it or leak into the effective config.
  it("zděděné `email: true` v defaults se ignoruje", () => {
    writeDefaults({ version: 1, notify: { email: true, inApp: true } })
    expect(getNotifySettings()).toEqual({ inApp: true })
  })

  it("zděděné `recipients` v uživatelském souboru se ignorují", () => {
    writeUser({ notify: { email: true, inApp: true, recipients: ["sef@firma.cz"] } })
    expect(getNotifySettings()).toEqual({ inApp: true })
  })
})

describe("writeNotifyOverride", () => {
  it("zapíše sparse override a nedotkne se shipped defaults", async () => {
    const { writeNotifyOverride } = await import("./notify-settings")
    writeDefaults({ version: 1, notify: { inApp: true } })
    writeNotifyOverride({ inApp: false })
    expect(getNotifySettings()).toEqual({ inApp: false })
    // shipped defaults untouched
    expect(JSON.parse(readFileSync(process.env.SID_DEFAULTS_FILE!, "utf8")).notify.inApp).toBe(true)
  })

  it("opakovaný zápis slučuje, nepřepisuje celý blok", async () => {
    const { writeNotifyOverride } = await import("./notify-settings")
    writeNotifyOverride({ inApp: false })
    writeNotifyOverride({ neznamy: 1 })
    expect(getNotifySettings()).toEqual({ inApp: false })
  })

  it("soubor s override má práva 0600", async () => {
    const { writeNotifyOverride, userSettingsPath } = await import("./notify-settings")
    writeNotifyOverride({ inApp: false })
    expect(statSync(userSettingsPath()).mode & 0o777).toBe(0o600)
  })
})
