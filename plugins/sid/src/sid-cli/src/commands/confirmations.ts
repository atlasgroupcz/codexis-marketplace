import { parseArgs } from "node:util"
import { apiGet, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"
import { confirmationsUrl, documentUrl, resolveRole } from "../links"

const USAGE = "Použití: sid-cli confirmations list [--state pending|confirmed]"

/**
 * Read-only on purpose: confirming is a legal acknowledgement and belongs in
 * SID with the document on screen, not behind a chat message.
 */
export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const [sub, ...rest] = argv

  if (sub === "confirm") {
    throw new CliError(
      "Potvrzení se provádí v aplikaci SID, ne z příkazové řádky — " +
        "otevřete dokument v SID a potvrďte ho tam. " +
        `Seznam čekajících: ${confirmationsUrl(await resolveRole(f))}`,
      EXIT.USAGE,
    )
  }

  if (sub === "list") {
    const { values } = parseArgs({ args: rest, options: { state: { type: "string" } } })
    const qs = values.state ? `?state=${encodeURIComponent(values.state)}` : ""
    const items = (await apiGet(`/api/v1/confirmations${qs}`, f)) as { documentId?: string }[]
    const role = await resolveRole(f)
    return {
      confirmations: items.map((c) => (c.documentId ? { ...c, url: documentUrl(c.documentId, role) } : c)),
      confirmInApp: confirmationsUrl(role),
    }
  }

  throw new CliError(USAGE, EXIT.USAGE)
}
