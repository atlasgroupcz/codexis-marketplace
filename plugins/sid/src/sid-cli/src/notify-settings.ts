import { readFileSync, writeFileSync, mkdirSync, renameSync } from "node:fs"
import { join, dirname } from "node:path"
import { homedir } from "node:os"
import { userHome } from "./config"

/**
 * Notification channel settings — same convention as the znamky, katastr,
 * insolvence and codexis plugins:
 *
 *   shipped defaults → ~/.local/share/sid/settings/defaults.json
 *                      (placed there by the install hook)
 *   user override    → $CODEXIS_PUBLIC_USER_HOME/.cdx/apps/sid/settings.json
 *                      (sparse — only the changed keys)
 *
 * The effective config is composed at read time; nothing is merged back into
 * the user file.
 *
 * The bell is the only channel. Unlike the peers — which watch external
 * registries that have no notifications of their own — SID mails its users
 * itself (per-event templates, a link to the document, per-tenant gating),
 * so a second mailer here would only duplicate it with a worse message. An
 * `email` / `recipients` block left over from an older version of the
 * plugin is read and dropped.
 */

export type NotifySettings = { inApp: boolean }

const BUILTIN: NotifySettings = { inApp: true }

function defaultsPath(): string {
  // SID_DEFAULTS_FILE is a test-only override.
  return process.env.SID_DEFAULTS_FILE || join(homedir(), ".local", "share", "sid", "settings", "defaults.json")
}

export function userSettingsPath(): string {
  return join(userHome(), ".cdx", "apps", "sid", "settings.json")
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
    // Missing or malformed file falls back to what we have — a typo in the
    // config must not fail the scheduled run.
    return null
  }
}

function notifyBlock(source: Record<string, unknown> | null): Partial<NotifySettings> {
  const raw = source?.notify
  if (!raw || typeof raw !== "object") return {}
  const { inApp } = raw as Record<string, unknown>
  return typeof inApp === "boolean" ? { inApp } : {}
}

export function getNotifySettings(): NotifySettings {
  return {
    ...BUILTIN,
    ...notifyBlock(readJson(defaultsPath())),
    ...notifyBlock(readJson(userSettingsPath())),
  }
}
