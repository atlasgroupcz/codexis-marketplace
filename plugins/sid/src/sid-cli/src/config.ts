import { readFileSync, writeFileSync, mkdirSync, renameSync, rmSync, existsSync } from "node:fs"
import { join, dirname } from "node:path"
import { homedir } from "node:os"

/** Stable contract — SKILL.md documents these and the automation relies on them. */
export const EXIT = { OK: 0, ERROR: 1, USAGE: 2, AUTH: 3, NOT_FOUND: 4, RATE_LIMIT: 5 } as const

const DEFAULT_BASE_URL = "https://sid.codexis.cz"

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
 * Paths are appended as `${base}/api/…`, so a trailing slash would produce
 * `//api/…` — and a trailing slash is exactly what someone types by hand.
 */
export function getBaseUrl(): string {
  const raw = process.env.CODEXIS_PLUGIN_SID_API_URL || readEnvFile().SID_API_URL || DEFAULT_BASE_URL
  return raw.trim().replace(/\/+$/, "")
}

/** Atomic write, mode 0600. Keys already in the file are preserved. */
function writeEnvFile(entries: Record<string, string>): void {
  const path = envFilePath()
  mkdirSync(dirname(path), { recursive: true, mode: 0o700 })
  const body = Object.entries(entries).map(([k, v]) => `${k}=${v}`).join("\n") + "\n"
  const tmp = `${path}.tmp-${process.pid}`
  writeFileSync(tmp, body, { mode: 0o600 })
  renameSync(tmp, path)
}

export function writeApiKey(key: string): void {
  writeEnvFile({ ...readEnvFile(), SID_API_KEY: key })
}

export function deleteApiKey(): void {
  const rest = readEnvFile()
  delete rest.SID_API_KEY
  if (Object.keys(rest).length === 0) {
    if (existsSync(envFilePath())) rmSync(envFilePath())
    return
  }
  writeEnvFile(rest)
}
