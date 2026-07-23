import { readFileSync, writeFileSync, mkdirSync, renameSync } from "node:fs"
import { join, dirname } from "node:path"
import { homedir } from "node:os"
import { userHome } from "./config"

/**
 * Notification channel settings - same convention as the znamky, katastr,
 * insolvence and codexis plugins:
 *
 *   shipped defaults → ~/.local/share/sid/settings/defaults.json
 *                      (placed there by the install hook)
 *   user override    → $CODEXIS_PUBLIC_USER_HOME/.cdx/apps/sid/settings.json
 *                      (sparse - only the changed keys)
 *
 * The effective config is composed at read time; nothing is merged back into
 * the user file.
 *
 * E-mail is off by default, matching the peers - a daily automation should
 * not reach someone's inbox unasked.
 */

export type NotifySettings = { email: boolean; inApp: boolean; recipients: string[] }

const BUILTIN: NotifySettings = { email: false, inApp: true, recipients: [] }

function defaultsPath(): string {
  // SID_DEFAULTS_FILE is a test-only override.
  return process.env.SID_DEFAULTS_FILE || join(homedir(), ".local", "share", "sid", "settings", "defaults.json")
}

export function userSettingsPath(): string {
  return join(userHome(), ".cdx", "apps", "sid", "settings.json")
}

/** Deliberately loose - the mail server validates properly anyway. */
export function isEmail(value: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(value.trim())
}

/**
 * Writes the sparse user override, merging into whatever the file already
 * holds so saving one toggle doesn't clear the others. Shipped defaults are
 * never modified.
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
    // Missing or malformed file falls back to what we have - a typo in the
    // config must not fail the scheduled run.
    return null
  }
}

/** Extra recipients - non-empty strings only, trimmed and de-duplicated (case-insensitive). */
function cleanRecipients(raw: unknown): string[] | undefined {
  if (!Array.isArray(raw)) return undefined
  const seen = new Set<string>()
  const out: string[] = []
  for (const v of raw) {
    if (typeof v !== "string") continue
    const t = v.trim()
    if (!t) continue
    const k = t.toLowerCase()
    if (seen.has(k)) continue
    seen.add(k)
    out.push(t)
  }
  return out
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
