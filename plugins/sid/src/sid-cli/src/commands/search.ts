import { parseArgs } from "node:util"
import { apiGet, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"
import { documentUrl, resolveRole } from "../links"

export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const { values, positionals } = parseArgs({
    args: argv,
    allowPositionals: true,
    options: { size: { type: "string", default: "10" }, from: { type: "string" } },
  })
  const q = positionals.join(" ").trim()
  if (!q) throw new CliError("Použití: sid-cli search <dotaz> [--size N] [--from N]", EXIT.USAGE)

  const sp = new URLSearchParams({ q, size: String(values.size) })
  if (values.from) sp.set("from", String(values.from))

  const res = (await apiGet(`/api/search?${sp}`, f)) as {
    total: number
    hits?: { documentId?: string }[]
    source: string
  }
  const role = await resolveRole(f)
  // Guard the array: on a contract drift where `hits` is missing, `.map` would
  // throw a generic error and swallow the opensearch_unavailable warning below.
  const hits = (res.hits ?? []).map((h) => (h.documentId ? { ...h, url: documentUrl(h.documentId, role) } : h))

  // `total` is recomputed after the ACL filter, so it never exceeds `size`.
  if (res.source === "opensearch_unavailable") {
    return {
      ...res,
      hits,
      warning:
        "Fulltext je dočasně nedostupný (opensearch_unavailable) - prázdný výsledek NEZNAMENÁ, " +
        "že nic neexistuje. Zkuste to později nebo použijte `sid-cli docs list --query`.",
    }
  }
  return { ...res, hits }
}
