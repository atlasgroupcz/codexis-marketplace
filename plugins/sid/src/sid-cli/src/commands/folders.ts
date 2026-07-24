import { apiGet, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"
import { folderUrl, resolveRole } from "../links"

const USAGE = "Použití: sid-cli folders list"

type Folder = {
  id: string
  name: string
  parentId: string | null
  documentCount?: number
}

/**
 * "Směrnice / Bezpečnost" - a bare id means nothing to a person, and the API
 * returns a flat list with `parentId` only.
 */
function pathOf(folder: Folder, byId: Map<string, Folder>): string {
  const names: string[] = []
  let current: Folder | undefined = folder
  const seen = new Set<string>()
  while (current && !seen.has(current.id)) {
    seen.add(current.id)
    names.unshift(current.name)
    current = current.parentId ? byId.get(current.parentId) : undefined
  }
  return names.join(" / ")
}

export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  if (argv[0] !== "list") throw new CliError(USAGE, EXIT.USAGE)

  const folders = (await apiGet("/api/v1/folders", f)) as Folder[]
  const byId = new Map(folders.map((x) => [x.id, x]))
  const role = await resolveRole(f)

  return {
    folders: folders
      .map((x) => ({ ...x, path: pathOf(x, byId), url: folderUrl(x.id, role) }))
      .sort((a, b) => a.path.localeCompare(b.path, "cs")),
  }
}
