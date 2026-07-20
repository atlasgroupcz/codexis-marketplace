import { readFileSync } from "node:fs"
import { join } from "node:path"
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

export type NotifySettings = { email: boolean; inApp: boolean }

const BUILTIN: NotifySettings = { email: false, inApp: true }

function defaultsPath(): string {
  // SID_DEFAULTS_FILE je override pro testy.
  return process.env.SID_DEFAULTS_FILE || join(homedir(), ".local", "share", "sid", "settings", "defaults.json")
}

function userPath(): string {
  return join(userHome(), ".cdx", "apps", "sid", "settings.json")
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

function notifyBlock(source: Record<string, unknown> | null): Partial<NotifySettings> {
  const raw = source?.notify
  if (!raw || typeof raw !== "object") return {}
  const out: Partial<NotifySettings> = {}
  const { email, inApp } = raw as Record<string, unknown>
  if (typeof email === "boolean") out.email = email
  if (typeof inApp === "boolean") out.inApp = inApp
  return out
}

export function getNotifySettings(): NotifySettings {
  return {
    ...BUILTIN,
    ...notifyBlock(readJson(defaultsPath())),
    ...notifyBlock(readJson(userPath())),
  }
}
