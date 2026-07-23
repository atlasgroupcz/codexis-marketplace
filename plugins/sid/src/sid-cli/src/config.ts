import { readFileSync, writeFileSync, mkdirSync, renameSync, rmSync, existsSync } from "node:fs"
import { join, dirname } from "node:path"
import { homedir } from "node:os"

/** Stable contract - SKILL.md documents these and the automation relies on them. */
export const EXIT = { OK: 0, ERROR: 1, USAGE: 2, AUTH: 3, NOT_FOUND: 4, RATE_LIMIT: 5 } as const

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

/** Atomic write, mode 0600. Keys already in the file are preserved. */
function writeEnvFile(entries: Record<string, string>): void {
  // A newline in any value would forge extra `.env` lines on the next read.
  // Callers validate their inputs; this is the last-line backstop.
  for (const [k, v] of Object.entries(entries)) {
    if (/[\r\n]/.test(k) || /[\r\n]/.test(v)) {
      throw new Error(`Odmítnut zápis do .env: hodnota '${k}' obsahuje konec řádku.`)
    }
  }
  const path = envFilePath()
  mkdirSync(dirname(path), { recursive: true, mode: 0o700 })
  const body = Object.entries(entries).map(([k, v]) => `${k}=${v}`).join("\n") + "\n"
  const tmp = `${path}.tmp-${process.pid}`
  writeFileSync(tmp, body, { mode: 0o600 })
  renameSync(tmp, path)
}

/** Sets a single key in the env file, leaving the others untouched. */
export function writeEnvValue(key: string, value: string): void {
  writeEnvFile({ ...readEnvFile(), [key]: value })
}

export function writeApiKey(key: string): void {
  writeEnvFile({ ...readEnvFile(), SID_API_KEY: key })
}

export function deleteApiKey(): void {
  const rest = readEnvFile()
  delete rest.SID_API_KEY
  // The cached role/e-mail belong to the removed key - leaving them would let
  // `settings status` report a stale role/isAdmin for a config that is gone.
  delete rest.SID_USER_ROLE
  delete rest.SID_USER_EMAIL
  if (Object.keys(rest).length === 0) {
    if (existsSync(envFilePath())) rmSync(envFilePath())
    return
  }
  writeEnvFile(rest)
}
