import { spawnSync } from "node:child_process"
import { existsSync } from "node:fs"
import { join } from "node:path"

/**
 * Store a plugin setting through `cdxctl appenv`.
 *
 * `~/.cdx/apps/sid/.env` is read-only inside the plugin's isolation, so the
 * daemon rewrites it on the plugin's behalf. Reading it stays in `config.ts`.
 */

/** `cdxctl` exited nonzero or could not be started. */
export class AppEnvError extends Error {
  readonly exitCode: number | null
  constructor(message: string, exitCode: number | null) {
    super(message)
    this.name = "AppEnvError"
    this.exitCode = exitCode
  }
}

/** SID_CDXCTL is a test-only override. */
function cdxctlProgram(): string {
  if (process.env.SID_CDXCTL) return process.env.SID_CDXCTL
  for (const home of [process.env.HOME, process.env.CODEXIS_PUBLIC_USER_HOME]) {
    if (!home) continue
    const candidate = join(home, ".local", "bin", "cdxctl")
    if (existsSync(candidate)) return candidate
  }
  // Resolved from PATH by the spawn itself.
  return "cdxctl"
}

function run(args: string[]): void {
  // The value is an argv element, never a shell string and never logged.
  const result = spawnSync(cdxctlProgram(), args, {
    stdio: ["ignore", "ignore", "pipe"],
    encoding: "utf8",
  })
  if (result.error) {
    throw new AppEnvError(`cdxctl se nepodařilo spustit: ${result.error.message}`, null)
  }
  if (result.status === 0) return
  const detail = (result.stderr ?? "").trim()
  throw new AppEnvError(`cdxctl appenv skončil s kódem ${result.status}: ${detail}`, result.status)
}

/** `cdxctl appenv <app> set <name> <value>`. */
export function setAppEnv(app: string, name: string, value: string): void {
  run(["appenv", app, "set", name, value])
}

/** `cdxctl appenv <app> delete <name>`. */
export function deleteAppEnv(app: string, name: string): void {
  run(["appenv", app, "delete", name])
}
