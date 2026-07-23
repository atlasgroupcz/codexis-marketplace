import { parseArgs } from "node:util"
import { apiGet, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"

const USAGE = "Použití: sid-cli attributes list [--all]"

type Attribute = {
  id: string
  name: string
  type: string
  options?: string[]
  archivedAt?: string | null
  documentCount?: number
  scopeFolderId?: string | null
}

/**
 * The attribute catalogue - the counterpart to `docs list --attribute`, which
 * needs ids the caller has no other way to learn.
 */
export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const [sub, ...rest] = argv
  if (sub !== "list") throw new CliError(USAGE, EXIT.USAGE)

  const { values } = parseArgs({ args: rest, options: { all: { type: "boolean", default: false } } })
  const all = (await apiGet("/api/v1/attributes", f)) as Attribute[]

  // Archived attributes can no longer be assigned, so they are noise when the
  // point is to build a filter.
  const attributes = values.all ? all : all.filter((a) => !a.archivedAt)

  return {
    attributes: attributes.map((a) => ({
      id: a.id,
      name: a.name,
      type: a.type,
      options: a.options ?? [],
      documentCount: a.documentCount ?? 0,
      ...(a.scopeFolderId ? { scopeFolderId: a.scopeFolderId } : {}),
      ...(a.archivedAt ? { archivedAt: a.archivedAt } : {}),
    })),
  }
}
