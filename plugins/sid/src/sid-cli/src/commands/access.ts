import { apiGet, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"
import { folderUrl, resolveRole } from "../links"
import type { Role } from "../links"

const USAGE =
  "Použití: sid-cli access user <email|jméno|id> | access group <název|id>"

// A principal can be named or given by id. An id skips the lookup entirely.
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

type User = { id: string; email?: string; name?: string }
type Group = { id: string; name?: string }
type PathPart = { id: string; name: string }
type UserFolder = { folderId: string; folderName: string; folderPath: PathPart[]; role: string; source: string }
type GroupFolder = { folderId: string; folderName: string; folderPath: PathPart[]; role: string; deny: boolean }

/** Exact, case-insensitive match; every hit is kept so ambiguity is an error, not a silent pick. */
function matchAll<T>(items: T[], pick: (x: T) => string | undefined, needle: string): T[] {
  const n = needle.trim().toLowerCase()
  return items.filter((x) => (pick(x) ?? "").trim().toLowerCase() === n)
}

async function resolveUser(needle: string, f?: FetchLike): Promise<User> {
  if (UUID.test(needle)) return { id: needle }
  const users = (await apiGet("/api/v1/users", f)) as User[]
  // E-mail is unique and unambiguous; fall back to the display name only if no
  // e-mail matched, since names repeat.
  const byEmail = matchAll(users, (u) => u.email, needle)
  const hits = byEmail.length > 0 ? byEmail : matchAll(users, (u) => u.name, needle)
  if (hits.length === 0) {
    throw new CliError(`Uživatel '${needle}' nenalezen — zadejte e-mail nebo id.`, EXIT.NOT_FOUND)
  }
  if (hits.length > 1) {
    const opts = hits.map((u) => u.email ?? u.id).join(", ")
    throw new CliError(`'${needle}' odpovídá více uživatelům — upřesněte e-mailem nebo id: ${opts}`, EXIT.USAGE)
  }
  return hits[0]!
}

async function resolveGroup(needle: string, f?: FetchLike): Promise<Group> {
  if (UUID.test(needle)) return { id: needle }
  const groups = (await apiGet("/api/v1/groups", f)) as Group[]
  const hits = matchAll(groups, (g) => g.name, needle)
  if (hits.length === 0) throw new CliError(`Skupina '${needle}' nenalezena.`, EXIT.NOT_FOUND)
  if (hits.length > 1) {
    throw new CliError(`'${needle}' odpovídá více skupinám — upřesněte id: ${hits.map((g) => g.id).join(", ")}`, EXIT.USAGE)
  }
  return hits[0]!
}

/** A readable folder path, falling back to the bare name if the path is empty. */
function folderLabel(x: { folderPath: PathPart[]; folderName: string }): string {
  return x.folderPath.map((p) => p.name).join(" / ") || x.folderName
}

const EMPTY_NOTE =
  "Žádné složky: principal nikde nemá přístup, nebo na dané složky nemáte admin právo. Práva se upravují v SID."

export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const [sub, needle] = argv
  if ((sub !== "user" && sub !== "group") || !needle) throw new CliError(USAGE, EXIT.USAGE)

  const role: Role | null = await resolveRole(f)

  if (sub === "user") {
    const u = await resolveUser(needle, f)
    const res = (await apiGet(`/api/v1/access/users/${encodeURIComponent(u.id)}`, f)) as {
      folders: UserFolder[]
    }
    const folders = res.folders.map((x) => ({
      folder: folderLabel(x),
      role: x.role,
      source: x.source, // "přímý" or "👥 <skupina>"
      url: folderUrl(x.folderId, role),
    }))
    return {
      user: { id: u.id, ...(u.email ? { email: u.email } : {}), ...(u.name ? { name: u.name } : {}) },
      folders,
      ...(folders.length === 0 ? { note: EMPTY_NOTE } : {}),
    }
  }

  const g = await resolveGroup(needle, f)
  const res = (await apiGet(`/api/v1/access/groups/${encodeURIComponent(g.id)}`, f)) as {
    folders: GroupFolder[]
  }
  const folders = res.folders.map((x) => ({
    folder: folderLabel(x),
    role: x.role,
    ...(x.deny ? { deny: true } : {}),
    url: folderUrl(x.folderId, role),
  }))
  return {
    group: { id: g.id, ...(g.name ? { name: g.name } : {}) },
    folders,
    ...(folders.length === 0 ? { note: EMPTY_NOTE } : {}),
  }
}
