import { parseArgs } from "node:util"
import { apiGet, type FetchLike } from "../http"
import { documentUrl, resolveRole } from "../links"

export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const { values } = parseArgs({
    args: argv,
    options: { all: { type: "boolean", default: false }, limit: { type: "string", default: "50" } },
  })
  const sp = new URLSearchParams()
  if (!values.all) sp.set("unreadOnly", "true")
  sp.set("limit", String(values.limit))
  const items = (await apiGet(`/api/notifications?${sp}`, f)) as { documentId?: string }[]
  // Attach a link into SID, like every sibling command — a notification about
  // a document should lead the user to it, not dead-end as raw JSON.
  const role = await resolveRole(f)
  return items.map((n) => (n.documentId ? { ...n, url: documentUrl(n.documentId, role) } : n))
}
