import { readFileSync } from "node:fs"
import { join } from "node:path"
import { homedir } from "node:os"

import { setAppEnv, deleteAppEnv } from "./app-env"

/** The app the settings belong to, as `cdxctl appenv` names it. */
const APP_ID = "sid"

/**
 * Stable contract - SKILL.md documents these and the automation relies on them.
 *
 * AUTH is strictly "the key won't authenticate" (401). FORBIDDEN is separate
 * because SID answers an unknown id with 403 rather than 404 (it refuses to
 * confirm which ids exist), so the two need different advice: renew the key
 * vs. check the id.
 */
export const EXIT = {
  OK: 0,
  ERROR: 1,
  USAGE: 2,
  AUTH: 3,
  NOT_FOUND: 4,
  RATE_LIMIT: 5,
  FORBIDDEN: 6,
} as const

// Fallback for runs outside the daemon. The manifest (plugin.json) injects
// CODEXIS_PLUGIN_SID_API_URL - production (sid.codexis.cz) - which always wins
// via the resolver below; this onprem default only applies when neither the
// env var nor a .env override is present.
const DEFAULT_BASE_URL = "https://sid.onprem.agrp.dev"

export function userHome(): string {
  return process.env.CODEXIS_PUBLIC_USER_HOME || homedir()
}

/** SID_ENV_FILE is a test-only override. */
export function envFilePath(): string {
  return process.env.SID_ENV_FILE || join(userHome(), ".cdx", "apps", "sid", ".env")
}

export function readEnvFile(): Record<string, string> {
  const out: Record<string, string> = {}
  try {
    for (const line of readFileSync(envFilePath(), "utf8").split("\n")) {
      const t = line.trim()
      if (!t || t.startsWith("#") || !t.includes("=")) continue
      const i = t.indexOf("=")
      out[t.slice(0, i).trim()] = t.slice(i + 1).trim()
    }
  } catch {
    /* Missing file means "not configured yet", not a failure. */
  }
  return out
}

export function getApiKey(): string {
  return readEnvFile().SID_API_KEY ?? ""
}

/**
 * A plausible API key: non-empty and free of whitespace. The point is to
 * reject a pasted value carrying a newline, which would otherwise be written
 * as an extra `.env` line (e.g. injecting SID_API_URL and repointing every
 * request). Deliberately loose about the exact charset - SID validates the
 * key itself, and an unusual-but-valid charset (base64url `-`/`_`) must pass.
 */
export function isPlausibleApiKey(key: string): boolean {
  return key.length > 0 && !/\s/.test(key)
}

/**
 * Paths are appended as `${base}/api/…`, so a trailing slash would produce
 * `//api/…` - and a trailing slash is exactly what someone types by hand.
 */
export function getBaseUrl(): string {
  const raw = process.env.CODEXIS_PLUGIN_SID_API_URL || readEnvFile().SID_API_URL || DEFAULT_BASE_URL
  return raw.trim().replace(/\/+$/, "")
}

/**
 * A value that would forge extra `.env` lines once the daemon writes it.
 * Callers validate their inputs; this is the last-line backstop.
 */
function rejectNewlines(key: string, value: string): void {
  if (/[\r\n]/.test(key) || /[\r\n]/.test(value)) {
    throw new Error(`Odmítnuto uložení: hodnota '${key}' obsahuje konec řádku.`)
  }
}

/** Sets a single key through the platform, leaving the others untouched. */
export function writeEnvValue(key: string, value: string): void {
  rejectNewlines(key, value)
  setAppEnv(APP_ID, key, value)
}

export function writeApiKey(key: string): void {
  writeEnvValue("SID_API_KEY", key)
}

export function deleteApiKey(): void {
  const stored = readEnvFile()
  // The cached role/e-mail belong to the removed key - leaving them would let
  // `settings status` report a stale role/isAdmin for a config that is gone.
  for (const name of ["SID_API_KEY", "SID_USER_ROLE", "SID_USER_EMAIL"]) {
    if (name in stored) deleteAppEnv(APP_ID, name)
  }
}
