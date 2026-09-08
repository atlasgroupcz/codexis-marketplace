import { chmodSync, existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"

/**
 * A fake `cdxctl` for the tests.
 *
 * Writing a setting now runs `cdxctl appenv`, so what a test can observe is the
 * command line the plugin asked with. The script records its argv, one line per
 * run, and exits with whatever `failWith` last stored.
 *
 * `bunfig.toml` preloads `test-preload.ts`, which installs one of these for the
 * whole run - so a test that only writes as a side effect needs no setup. A test
 * that asserts on the calls makes its own and calls `restore()` afterwards.
 */
export class FakeCdxctl {
  private readonly log: string
  private readonly exitFile: string
  private readonly previous: string | undefined

  constructor(dir: string = mkdtempSync(join(tmpdir(), "sid-cdxctl-"))) {
    this.log = join(dir, "cdxctl-calls.log")
    this.exitFile = join(dir, "cdxctl-exit")
    const script = join(dir, "cdxctl")
    writeFileSync(
      script,
      "#!/bin/sh\n" +
        `printf '%s\\n' "$*" >> '${this.log}'\n` +
        `code=$(cat '${this.exitFile}' 2>/dev/null || echo 0)\n` +
        '[ "$code" -eq 0 ] || echo "fake failure" >&2\n' +
        'exit "$code"\n',
    )
    chmodSync(script, 0o755)
    this.previous = process.env.SID_CDXCTL
    process.env.SID_CDXCTL = script
  }

  /** One line per invocation: the argv joined by spaces. */
  calls(): string[] {
    if (!existsSync(this.log)) return []
    return readFileSync(this.log, "utf8").trimEnd().split("\n").filter(Boolean)
  }

  failWith(code: number): void {
    writeFileSync(this.exitFile, String(code))
  }

  /** Hand `SID_CDXCTL` back, so a removed temp dir cannot outlive its test. */
  restore(): void {
    if (this.previous === undefined) delete process.env.SID_CDXCTL
    else process.env.SID_CDXCTL = this.previous
  }
}
