import { parseArgs } from "node:util"
import { apiGet, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"

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

  const res = (await apiGet(`/api/search?${sp}`, f)) as { total: number; hits: unknown[]; source: string }

  // `total` is recomputed after the ACL filter, so it never exceeds `size`.
  if (res.source === "opensearch_unavailable") {
    return {
      ...res,
      warning:
        "Fulltext je dočasně nedostupný (opensearch_unavailable) — prázdný výsledek NEZNAMENÁ, " +
        "že nic neexistuje. Zkuste to později nebo použijte `sid-cli docs list --query`.",
    }
  }
  return res
}
