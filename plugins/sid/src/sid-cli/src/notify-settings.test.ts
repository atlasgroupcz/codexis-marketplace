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
  const dir = join(home, ".cdx", "apps", "sid")
  mkdirSync(dir, { recursive: true })
  writeFileSync(join(dir, "settings.json"), JSON.stringify(obj))
}

describe("getNotifySettings", () => {
  it("bez jakýchkoli souborů: e-mail vypnutý, bell zapnutý (vzor ostatních pluginů)", () => {
    expect(getNotifySettings()).toEqual({ email: false, inApp: true, recipients: [] })
  })

  it("čte shipped defaults", () => {
    writeDefaults({ version: 1, notify: { email: true, inApp: false } })
    expect(getNotifySettings()).toEqual({ email: true, inApp: false, recipients: [] })
  })

  it("uživatelský override přebije defaults, a to jen v uvedených klíčích", () => {
    writeDefaults({ version: 1, notify: { email: false, inApp: true } })
    writeUser({ notify: { email: true } })
    expect(getNotifySettings()).toEqual({ email: true, inApp: true, recipients: [] })
  })

  it("poškozený JSON nespadne, použije vestavěné hodnoty", () => {
    writeFileSync(process.env.SID_DEFAULTS_FILE!, "{ tohle není json")
    expect(getNotifySettings()).toEqual({ email: false, inApp: true, recipients: [] })
  })

  it("nesmyslné typy ignoruje", () => {
    writeDefaults({ notify: { email: "ano", inApp: 1 } })
    expect(getNotifySettings()).toEqual({ email: false, inApp: true, recipients: [] })
  })
})

describe("recipients", () => {
  it("bez konfigurace je seznam prázdný", () => {
    expect(getNotifySettings().recipients).toEqual([])
  })

  it("uživatel může přidat další adresáty", () => {
    writeUser({ notify: { recipients: ["sef@firma.cz", "kolega@firma.cz"] } })
    expect(getNotifySettings().recipients).toEqual(["sef@firma.cz", "kolega@firma.cz"])
  })

  it("nesmyslné hodnoty vyfiltruje (jen neprázdné řetězce)", () => {
    writeUser({ notify: { recipients: ["a@b.cz", "", 42, null, "  c@d.cz  "] } })
    expect(getNotifySettings().recipients).toEqual(["a@b.cz", "c@d.cz"])
  })

  it("recipients, které není pole, ignoruje", () => {
    writeUser({ notify: { recipients: "sef@firma.cz" } })
    expect(getNotifySettings().recipients).toEqual([])
  })

  it("zdvojené adresy deduplikuje case-insensitive, zachová první zápis", () => {
    writeUser({ notify: { recipients: ["Sef@firma.cz", "sef@firma.cz", "  SEF@FIRMA.CZ "] } })
    expect(getNotifySettings().recipients).toEqual(["Sef@firma.cz"])
  })
})

describe("writeNotifyOverride", () => {
  it("zapíše sparse override a nedotkne se shipped defaults", async () => {
    const { writeNotifyOverride } = await import("./notify-settings")
    writeDefaults({ version: 1, notify: { email: false, inApp: true, recipients: [] } })
    writeNotifyOverride({ email: true })
    expect(getNotifySettings()).toEqual({ email: true, inApp: true, recipients: [] })
    // shipped defaults untouched
    expect(JSON.parse(readFileSync(process.env.SID_DEFAULTS_FILE!, "utf8")).notify.email).toBe(false)
  })

  it("opakovaný zápis slučuje, nepřepisuje celý blok", async () => {
    const { writeNotifyOverride } = await import("./notify-settings")
    writeNotifyOverride({ email: true })
    writeNotifyOverride({ recipients: ["a@b.cz"] })
    expect(getNotifySettings()).toMatchObject({ email: true, recipients: ["a@b.cz"] })
  })

  it("soubor s override má práva 0600", async () => {
    const { writeNotifyOverride, userSettingsPath } = await import("./notify-settings")
    writeNotifyOverride({ email: true })
    expect(statSync(userSettingsPath()).mode & 0o777).toBe(0o600)
  })
})

describe("isEmail", () => {
  it("rozliší platnou a neplatnou adresu", async () => {
    const { isEmail } = await import("./notify-settings")
    expect(isEmail("sef@firma.cz")).toBe(true)
    expect(isEmail("a.b+c@sub.domena.co.uk")).toBe(true)
    expect(isEmail("bez-zavinace")).toBe(false)
    expect(isEmail("dve@@zavinace.cz")).toBe(false)
    expect(isEmail("mezera v@adrese.cz")).toBe(false)
  })
})
