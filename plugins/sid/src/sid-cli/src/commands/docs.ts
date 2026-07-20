import { parseArgs } from "node:util"
import { apiGet, apiPatch, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"
import { documentUrl, resolveRole } from "../links"

const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/

const USAGE =
  "Použití: sid-cli docs list [--query T] [--folder ID] [--author ID] [--attribute ID]… " +
  "[--from YYYY-MM-DD] [--to YYYY-MM-DD] [--type pdf|word|excel|image] [--personal] " +
  "[--sort newest|oldest|title|size] [--page N] [--page-size N]\n" +
  "         sid-cli docs get <id>\n" +
  "         sid-cli docs legislation <id>\n" +
  "         sid-cli docs set-validity <id> [--from YYYY-MM-DD] [--to YYYY-MM-DD] [--clear-from] [--clear-to]"

export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const [sub, ...rest] = argv

  if (sub === "get") {
    const id = rest[0]
    if (!id) throw new CliError(USAGE, EXIT.USAGE)
    const doc = (await apiGet(`/api/v1/documents/${encodeURIComponent(id)}`, f)) as { id?: string }
    return { ...doc, url: documentUrl(doc.id ?? id, await resolveRole(f)) }
  }

  if (sub === "legislation") {
    const id = rest[0]
    if (!id) throw new CliError(USAGE, EXIT.USAGE)
    const legislation = (await apiGet(
      `/api/v1/documents/${encodeURIComponent(id)}/legislation`,
      f,
    )) as unknown[]
    return { documentId: id, legislation }
  }

  if (sub === "set-validity") {
    const id = rest[0]
    if (!id) throw new CliError(USAGE, EXIT.USAGE)
    const { values } = parseArgs({
      args: rest.slice(1),
      options: {
        from: { type: "string" },
        to: { type: "string" },
        "clear-from": { type: "boolean", default: false },
        "clear-to": { type: "boolean", default: false },
      },
    })

    const patch: Record<string, string | null> = {}
    for (const [flag, field] of [["from", "effectiveFrom"], ["to", "effectiveTo"]] as const) {
      const raw = values[flag]
      if (raw === undefined) continue
      // Validate before the call so a Czech-style date fails as misuse
      // rather than as a server error.
      if (!ISO_DATE.test(raw)) {
        throw new CliError(`Datum musí být ve formátu YYYY-MM-DD, dostal jsem '${raw}'.`, EXIT.USAGE)
      }
      patch[field] = raw
    }
    if (values["clear-from"]) patch.effectiveFrom = null
    if (values["clear-to"]) patch.effectiveTo = null

    if (Object.keys(patch).length === 0) {
      throw new CliError(
        "Nebylo co nastavit — uveďte --from, --to, --clear-from nebo --clear-to.",
        EXIT.USAGE,
      )
    }
    return apiPatch(`/api/v1/documents/${encodeURIComponent(id)}`, patch, f)
  }

  if (sub === "list") {
    const { values } = parseArgs({
      args: rest,
      options: {
        query: { type: "string" },
        folder: { type: "string" },
        author: { type: "string" },
        sort: { type: "string" },
        page: { type: "string" },
        "page-size": { type: "string" },
        // Repeatable: the API ANDs the attributes together.
        attribute: { type: "string", multiple: true },
        from: { type: "string" },
        to: { type: "string" },
        type: { type: "string" },
        personal: { type: "boolean", default: false },
      },
    })
    const sp = new URLSearchParams()
    if (values.query) sp.set("query", values.query)
    if (values.folder) sp.set("folderId", values.folder)
    if (values.author) sp.set("authorId", values.author)
    if (values.sort) sp.set("sortBy", values.sort)
    if (values.from) sp.set("dateFrom", values.from)
    if (values.to) sp.set("dateTo", values.to)
    if (values.type) sp.set("mimeType", values.type)
    if (values.personal) sp.set("personalOnly", "true")
    if (values.page) sp.set("page", values.page)
    if (values["page-size"]) sp.set("pageSize", values["page-size"])
    for (const id of values.attribute ?? []) sp.append("attributeIds", id)
    const qs = sp.toString()
    const res = (await apiGet(`/api/v1/documents${qs ? `?${qs}` : ""}`, f)) as {
      documents?: { id?: string }[]
    }
    const role = await resolveRole(f)
    return {
      ...res,
      documents: (res.documents ?? []).map((d) => (d.id ? { ...d, url: documentUrl(d.id, role) } : d)),
    }
  }

  throw new CliError(USAGE, EXIT.USAGE)
}
