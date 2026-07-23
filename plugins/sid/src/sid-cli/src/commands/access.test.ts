import { describe, it, expect, beforeEach } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { run } from "./access"
import type { CliError } from "../http"

beforeEach(() => {
  const dir = mkdtempSync(join(tmpdir(), "sid-access-"))
  process.env.SID_ENV_FILE = join(dir, ".env")
  writeFileSync(
    process.env.SID_ENV_FILE,
    "SID_API_KEY=k\nSID_API_URL=http://s\nSID_USER_ROLE=ADMIN\n",
    { mode: 0o600 },
  )
  delete process.env.CODEXIS_PLUGIN_SID_API_URL
})

const USERS = [
  { id: "u-anna", email: "anna@firma.cz", name: "Anna Nová", role: "USER" },
  { id: "u-petr", email: "petr@firma.cz", name: "Petr Svoboda", role: "USER" },
  { id: "u-dup1", email: "dup1@firma.cz", name: "Jan Duplicitní", role: "USER" },
  { id: "u-dup2", email: "dup2@firma.cz", name: "Jan Duplicitní", role: "USER" },
]
const GROUPS = [
  { id: "g-hr", name: "HR" },
  { id: "g-it", name: "IT" },
]
const USER_ACCESS = {
  userId: "u-anna",
  folders: [
    {
      folderId: "f1",
      folderName: "Bezpečnost",
      folderPath: [{ id: "root", name: "Směrnice" }, { id: "f1", name: "Bezpečnost" }],
      role: "VIEWER",
      source: "přímý",
    },
  ],
}
const GROUP_ACCESS = {
  groupId: "g-hr",
  folders: [
    {
      permissionId: "p1",
      folderId: "f2",
      folderName: "Mzdy",
      folderPath: [{ id: "f2", name: "Mzdy" }],
      role: "EDITOR",
      deny: false,
    },
  ],
}

/** Route the fake fetch by path so one command can hit several endpoints. */
function router(cap: { urls: string[] } = { urls: [] }): typeof fetch {
  return (async (u: unknown) => {
    const url = String(u)
    cap.urls.push(url)
    const body = url.includes("/api/v1/users/me")
      ? { id: "me", role: "ADMIN" }
      : url.includes("/api/v1/users")
        ? USERS
        : url.includes("/api/v1/groups")
          ? GROUPS
          : url.includes("/api/v1/access/users/")
            ? USER_ACCESS
            : url.includes("/api/v1/access/groups/")
              ? GROUP_ACCESS
              : null
    return new Response(JSON.stringify(body), { status: body === null ? 404 : 200 })
  }) as unknown as typeof fetch
}

describe("access user", () => {
  it("resolves an email to the user and reports their folders with paths + links", async () => {
    const cap = { urls: [] as string[] }
    const out = (await run(["user", "anna@firma.cz"], router(cap))) as {
      user: { id: string; email: string }
      folders: Array<{ folder: string; role: string; source: string; url: string }>
    }
    expect(out.user.id).toBe("u-anna")
    expect(cap.urls.some((u) => u === "http://s/api/v1/access/users/u-anna")).toBe(true)
    expect(out.folders[0]!.folder).toBe("Směrnice / Bezpečnost")
    expect(out.folders[0]!.role).toBe("VIEWER")
    expect(out.folders[0]!.url).toBe("http://s/admin/files?folderId=f1")
  })

  it("víceslovné jméno bez uvozovek se spojí, ne zkrátí na první token", async () => {
    const out = (await run(["user", "Anna", "Nová"], router())) as { user: { id: string } }
    expect(out.user.id).toBe("u-anna")
  })

  it("a bare id skips the lookup and queries access directly", async () => {
    const cap = { urls: [] as string[] }
    await run(["user", "550e8400-e29b-41d4-a716-446655440000"], router(cap))
    expect(cap.urls.some((u) => u.includes("/api/v1/users") && !u.includes("/access/"))).toBe(false)
    expect(cap.urls.some((u) => u.includes("/api/v1/access/users/550e8400"))).toBe(true)
  })

  it("an unknown user is a not-found error, not an empty report", async () => {
    const err = (await run(["user", "nikdo@firma.cz"], router()).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(4) // NOT_FOUND
  })

  it("an ambiguous name refuses rather than guessing", async () => {
    const err = (await run(["user", "Jan Duplicitní"], router()).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2) // USAGE
    expect(err.message).toContain("více uživatel")
  })
})

describe("access group", () => {
  it("resolves a group name and reports its folders", async () => {
    const cap = { urls: [] as string[] }
    const out = (await run(["group", "HR"], router(cap))) as {
      group: { id: string; name: string }
      folders: Array<{ folder: string; role: string; url: string }>
    }
    expect(out.group.id).toBe("g-hr")
    expect(cap.urls.some((u) => u === "http://s/api/v1/access/groups/g-hr")).toBe(true)
    expect(out.folders[0]!.folder).toBe("Mzdy")
    expect(out.folders[0]!.role).toBe("EDITOR")
    expect(out.folders[0]!.url).toBe("http://s/admin/files?folderId=f2")
  })
})

describe("access usage", () => {
  it("missing principal → USAGE", async () => {
    const err = (await run(["user"], router()).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
  })
  it("unknown subcommand → USAGE", async () => {
    const err = (await run(["wat", "x"], router()).catch((e: unknown) => e)) as CliError
    expect(err.exitCode).toBe(2)
  })
})
