import { parseArgs } from "node:util"
import { apiGet, type FetchLike } from "../http"

export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const { values } = parseArgs({
    args: argv,
    options: { all: { type: "boolean", default: false }, limit: { type: "string", default: "50" } },
  })
  const sp = new URLSearchParams()
  if (!values.all) sp.set("unreadOnly", "true")
  sp.set("limit", String(values.limit))
  return apiGet(`/api/notifications?${sp}`, f)
}
