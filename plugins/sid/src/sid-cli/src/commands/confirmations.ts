import { parseArgs } from "node:util"
import { apiGet, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"
import { confirmationsUrl, documentUrl, resolveRole } from "../links"

const USAGE = "Použití: sid-cli confirmations list [--state pending|confirmed]"

/**
 * Read-only on purpose.
 *
 * Confirming a document is a legal acknowledgement ("I have read this"), so it
 * belongs in SID where the document is actually on screen — not in a chat
 * where it could be clicked through without ever opening the file. The CLI
 * reports what is pending and links to the app.
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
