import { readFileSync } from "node:fs"
import { join } from "node:path"

export const APP_ID = "turboleal"
export const API_KEY_VAR = "TURBOLEAL_API_KEY"
export const API_URL_VAR = "CODEXIS_PLUGIN_TURBOLEAL_API_URL"
export const DEFAULT_API_URL = "https://turboleal-profidata-main.onprem.agrp.dev"

/** An unset or blank variable reads as missing, like cdx_common::config::read_env. */
export function readEnv(name: string): string | undefined {
  const value = process.env[name]?.trim()
  return value ? value : undefined
}

/** The user's home as cdx_common::paths::user_home resolves it inside the chat VM. */
export function userHome(): string {
  return readEnv("CODEXIS_PUBLIC_USER_HOME") ?? readEnv("CDX_USER_HOME") ?? process.env.HOME ?? "/"
}

/** `<user home>/.cdx/apps/turboleal/.env`, written by the daemon through `cdxctl appenv`. */
export function envPath(): string {
  return join(userHome(), ".cdx", "apps", APP_ID, ".env")
}

/** One key of a `KEY=VALUE` file; blank lines, comments and empty values are skipped. */
export function readEnvVar(path: string, key: string): string | undefined {
  let contents: string
  try {
    contents = readFileSync(path, "utf8")
  } catch {
    return undefined
  }
  for (const raw of contents.split("\n")) {
    const line = raw.trim()
    if (!line || line.startsWith("#")) continue
    const at = line.indexOf("=")
    if (at < 0) continue
    if (line.slice(0, at).trim() !== key) continue
    const value = line.slice(at + 1).trim()
    return value || undefined
  }
  return undefined
}

export function apiUrl(): string {
  return (readEnv(API_URL_VAR) ?? DEFAULT_API_URL).replace(/\/+$/, "")
}

export function apiKey(): string | undefined {
  return readEnv(API_KEY_VAR) ?? readEnvVar(envPath(), API_KEY_VAR)
}
