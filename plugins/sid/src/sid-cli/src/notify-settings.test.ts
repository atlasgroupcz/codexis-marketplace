import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs"
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
  const dir = join(home, ".cdx", "apps", "sid")
  mkdirSync(dir, { recursive: true })
  writeFileSync(join(dir, "settings.json"), JSON.stringify(obj))
}

describe("getNotifySettings", () => {
  it("bez jakýchkoli souborů: e-mail vypnutý, bell zapnutý (vzor ostatních pluginů)", () => {
    expect(getNotifySettings()).toEqual({ email: false, inApp: true })
  })

  it("čte shipped defaults", () => {
    writeDefaults({ version: 1, notify: { email: true, inApp: false } })
    expect(getNotifySettings()).toEqual({ email: true, inApp: false })
  })

  it("uživatelský override přebije defaults, a to jen v uvedených klíčích", () => {
    writeDefaults({ version: 1, notify: { email: false, inApp: true } })
    writeUser({ notify: { email: true } })
    expect(getNotifySettings()).toEqual({ email: true, inApp: true })
  })

  it("poškozený JSON nespadne, použije vestavěné hodnoty", () => {
    writeFileSync(process.env.SID_DEFAULTS_FILE!, "{ tohle není json")
    expect(getNotifySettings()).toEqual({ email: false, inApp: true })
  })

  it("nesmyslné typy ignoruje", () => {
    writeDefaults({ notify: { email: "ano", inApp: 1 } })
    expect(getNotifySettings()).toEqual({ email: false, inApp: true })
  })
})
