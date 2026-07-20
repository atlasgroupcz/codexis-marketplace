import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./folders"
import type { CliError } from "../http"

beforeEach(() => {
  const dir = mkdtempSync(join(tmpdir(), "sid-folders-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  writeFileSync(process.env.SID_ENV_FILE, "SID_API_KEY=k\nSID_API_URL=http://s\nSID_USER_ROLE=USER\n", {
    mode: 0o600,
  })
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

const FOLDERS = [
  { id: "root", name: "Směrnice", parentId: null, documentCount: 3 },
  { id: "child", name: "Bezpečnost", parentId: "root", documentCount: 2 },
  { id: "other", name: "Smlouvy", parentId: null, documentCount: 0 },
]

function fetchFolders(cap: { url?: string } = {}): typeof fetch {
  return (async (u: unknown) => {
    cap.url = String(u)
    return new Response(JSON.stringify(FOLDERS), { status: 200 })
  }) as unknown as typeof fetch
}

describe("folders list", () => {
  it("calls the folders endpoint and returns each folder with its path", async () => {
    const cap: { url?: string } = {}
    const out = (await run(["list"], fetchFolders(cap))) as {
      folders: Array<{ id: string; path: string; url: string }>
    }
    expect(cap.url).toBe("http://s/api/v1/folders")
    // A flat list of ids is useless to the model — the path is what a person
    // recognises, and what makes `docs list --folder` usable.
    expect(out.folders.find((f) => f.id === "child")?.path).toBe("Směrnice / Bezpečnost")
    expect(out.folders.find((f) => f.id === "root")?.path).toBe("Směrnice")
  })

  it("each folder carries a link into the browser", async () => {
    const out = (await run(["list"], fetchFolders())) as { folders: Array<{ url: string }> }
    expect(out.folders[0]!.url).toBe("http://s/files?folderId=root")
  })

  it("unknown subcommand → USAGE", async () => {
    const err = (await run(["wat"]).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
  })
})
