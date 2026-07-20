import { parseArgs } from "node:util"
import { apiGet, apiPatch, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"

const USAGE =
  "Použití: sid-cli confirmations list [--state pending|confirmed] | sid-cli confirmations confirm <id>"

export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const [sub, ...rest] = argv

  if (sub === "list") {
    const { values } = parseArgs({ args: rest, options: { state: { type: "string" } } })
    const qs = values.state ? `?state=${encodeURIComponent(values.state)}` : ""
    return apiGet(`/api/v1/confirmations${qs}`, f)
  }

  if (sub === "confirm") {
    const id = rest[0]
    if (!id) throw new CliError(USAGE, EXIT.USAGE)
    return apiPatch(`/api/v1/confirmations/${encodeURIComponent(id)}`, { state: "CONFIRMED" }, f)
  }

  throw new CliError(USAGE, EXIT.USAGE)
}
