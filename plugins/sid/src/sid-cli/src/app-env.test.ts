import { describe, it, expect, beforeEach, afterEach } from "bun:test"
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { AppEnvError, setAppEnv, deleteAppEnv } from "./app-env"

let dir: string
let previousCdxctl: string | undefined
let previousHome: string | undefined
beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), "sid-appenv-"))
  previousCdxctl = process.env.SID_CDXCTL
  previousHome = process.env.HOME
})
afterEach(() => {
  if (previousCdxctl === undefined) delete process.env.SID_CDXCTL
  else process.env.SID_CDXCTL = previousCdxctl
  if (previousHome === undefined) delete process.env.HOME
  else process.env.HOME = previousHome
  rmSync(dir, { recursive: true, force: true })
})

/** A recording `cdxctl` at `path`. */
function fakeAt(path: string, log: string): void {
  mkdirSync(join(path, ".."), { recursive: true })
  writeFileSync(path, `#!/bin/sh\nprintf '%s\\n' "$*" >> '${log}'\n`)
  chmodSync(path, 0o755)
}

describe("app-env", () => {
  it("bez SID_CDXCTL sáhne po ~/.local/bin/cdxctl", () => {
    const log = join(dir, "calls.log")
    fakeAt(join(dir, ".local", "bin", "cdxctl"), log)
    delete process.env.SID_CDXCTL
    process.env.HOME = dir

    setAppEnv("sid", "SID_API_KEY", "sid_abc")

    expect(readFileSync(log, "utf8")).toBe("appenv sid set SID_API_KEY sid_abc\n")
  })

  it("chybějící cdxctl je chyba bez exit kódu", () => {
    process.env.SID_CDXCTL = join(dir, "neexistuje")

    const thrown = (() => {
      try {
        deleteAppEnv("sid", "SID_API_KEY")
      } catch (e) {
        return e as AppEnvError
      }
    })()

    expect(thrown).toBeInstanceOf(AppEnvError)
    expect(thrown!.exitCode).toBeNull()
    expect(thrown!.message).toContain("nepodařilo spustit")
  })
})
