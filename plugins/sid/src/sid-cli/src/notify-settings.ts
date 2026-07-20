import { readFileSync, writeFileSync, mkdirSync, renameSync } from "node:fs"
import { join, dirname } from "node:path"
import { homedir } from "node:os"
import { userHome } from "./config"

/**
 * Nastavení notifikačních kanálů — stejná konvence jako u pluginů znamky,
 * katastr, insolvence a codexis:
 *
 *   shipped defaults → ~/.local/share/sid/settings/defaults.json
 *                      (kopíruje tam install hook z plugins/sid/settings/sid/)
 *   uživatelský      → $CODEXIS_PUBLIC_USER_HOME/.cdx/apps/sid/settings.json
 *                      (sparse — jen změněné klíče)
 *
 * Výsledek se skládá při čtení; do uživatelského souboru se nic nesluje.
 *
 * E-mail je záměrně vypnutý ve výchozím stavu (shodně s ostatními pluginy) —
 * denní automatizace nemá bez vyžádání chodit uživateli do schránky.
 */

export type NotifySettings = { email: boolean; inApp: boolean; recipients: string[] }

const BUILTIN: NotifySettings = { email: false, inApp: true, recipients: [] }

function defaultsPath(): string {
  // SID_DEFAULTS_FILE je override pro testy.
  return process.env.SID_DEFAULTS_FILE || join(homedir(), ".local", "share", "sid", "settings", "defaults.json")
}

export function userSettingsPath(): string {
  return join(userHome(), ".cdx", "apps", "sid", "settings.json")
}

/** Zpřísněná, ale ne přehnaná validace — server stejně adresu ověří sám. */
export function isEmail(value: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(value.trim())
}

/**
 * Zapíše sparse override uživatele. Slučuje se s tím, co už v souboru je,
 * aby uložení jednoho přepínače nesmazalo ostatní; shipped defaults se
 * nikdy nemění.
 */
export function writeNotifyOverride(patch: Record<string, unknown>): void {
  const path = userSettingsPath()
  const current = readJson(path) ?? {}
  const currentNotify =
    current.notify && typeof current.notify === "object" ? (current.notify as Record<string, unknown>) : {}
  const next = { ...current, notify: { ...currentNotify, ...patch } }
  mkdirSync(dirname(path), { recursive: true, mode: 0o700 })
  const tmp = `${path}.tmp-${process.pid}`
  writeFileSync(tmp, JSON.stringify(next, null, 2) + "\n", { mode: 0o600 })
  renameSync(tmp, path)
}

function readJson(path: string): Record<string, unknown> | null {
  try {
    const parsed: unknown = JSON.parse(readFileSync(path, "utf8"))
    return parsed && typeof parsed === "object" ? (parsed as Record<string, unknown>) : null
  } catch {
    // Chybějící i poškozený soubor = použij, co máme; notifikace nesmí
    // shodit naplánovaný běh kvůli překlepu v konfiguraci.
    return null
  }
}

/** Adresáti navíc — jen neprázdné řetězce, ořezané. */
function cleanRecipients(raw: unknown): string[] | undefined {
  if (!Array.isArray(raw)) return undefined
  return raw.filter((v): v is string => typeof v === "string").map((v) => v.trim()).filter(Boolean)
}

function notifyBlock(source: Record<string, unknown> | null): Partial<NotifySettings> {
  const raw = source?.notify
  if (!raw || typeof raw !== "object") return {}
  const out: Partial<NotifySettings> = {}
  const { email, inApp, recipients } = raw as Record<string, unknown>
  if (typeof email === "boolean") out.email = email
  if (typeof inApp === "boolean") out.inApp = inApp
  const cleaned = cleanRecipients(recipients)
  if (cleaned) out.recipients = cleaned
  return out
}

export function getNotifySettings(): NotifySettings {
  return {
    ...BUILTIN,
    ...notifyBlock(readJson(defaultsPath())),
    ...notifyBlock(readJson(userSettingsPath())),
  }
}
