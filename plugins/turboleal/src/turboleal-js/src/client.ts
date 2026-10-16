import { apiKey as storedApiKey, apiUrl as configuredApiUrl } from "./config"
import { type DiffOverview, diffOverview } from "./diff"
import { alertPageUrl, czechDate, fold, isoDate, pragueDate, provisionTitle, sectionKey, typeLabel } from "./format"
import { cite, entry, type SourceEntry } from "./sources"

export type Me = { id: string; email: string | null; name: string | null }
export type WatchSet = { id: string; name: string; brief: string; extractionStatus: string; createdAt: string }
export type Section = { id: string; partId: string | null; label: string | null }
export type Document = {
  documentId: string
  citation: string
  title: string
  status: string
  wholeDocument: boolean
  sections: Section[]
}
export type Amendment = { documentId: string; citation: string }
export type Reference = {
  id: string
  text: string
  documentId: string | null
  partId: string | null
  kind: string
  reason: string | null
  fileName: string | null
  path: string | null
}
export type Alert = {
  id: string
  watchSetId: string
  type: string
  documentId: string
  citation: string
  sectionId: string
  partId: string | null
  label: string | null
  effectiveFrom: string | null
  detectedAt: string
  amendments: Amendment[]
  diff: string | null
  diffMarkdown: string | null
  references: Reference[]
  acknowledgedAt: string | null
  lawTitle: string | null
  turbolealUrl: string
}
export type Watched = {
  watchSetId: string
  watchSetName: string
  citation: string
  title: string
  wholeDocument: boolean
  sections: string[]
}
export type Summary = {
  type: string
  law: string | null
  citation: string
  section: string | null
  effectiveFrom: string | null
  inForce: boolean | null
  detectedAt: string | null
  amendments: string[]
  acknowledged: boolean
  turbolealUrl: string
  overview: DiffOverview | null
  diffMarkdown: string | null
}

type DateLike = string | Date
export type AlertQuery = {
  watchSet?: string
  unacknowledged?: boolean
  law?: string
  section?: string
  effectiveFrom?: DateLike
  effectiveTo?: DateLike
  detectedFrom?: DateLike
  detectedTo?: DateLike
  types?: string[]
}
export type WatchedQuery = { law?: string; section?: string }

type Page<T> = { items: T[]; nextCursor: string | null }
type Query = Record<string, string | number | boolean | undefined>

const PAGE_LIMIT = 200
const REQUEST_TIMEOUT_MS = 60_000
const HIDDEN_FIELDS = ["codexisUrl", "diffHtml", "remindedAt"]
const UNAUTHORIZED_HINT =
  "API klíč Turbolealu chybí nebo byl zneplatněn. Vytvořte nový v Turbolealu (API klíče) " +
  "a uložte ho přes `turboleal-cli auth login`."

export class TurbolealError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message)
    this.name = "TurbolealError"
  }
}

export type FetchLike = (input: URL, init: RequestInit) => Promise<Response>
export type ClientOptions = { apiUrl?: string; apiKey?: string; fetch?: FetchLike }

export class Turboleal {
  readonly appUrl: string
  private readonly key: string | undefined
  private readonly fetcher: FetchLike
  private readonly documentCache = new Map<string, Promise<Document[]>>()
  private watchSetCache: Promise<WatchSet[]> | undefined

  constructor(options: ClientOptions = {}) {
    this.appUrl = (options.apiUrl ?? configuredApiUrl()).replace(/\/+$/, "")
    this.key = options.apiKey ?? storedApiKey()
    this.fetcher = options.fetch ?? ((input, init) => fetch(input, init))
  }

  me(): Promise<Me> {
    return this.get("/me")
  }

  watchSets(): Promise<WatchSet[]> {
    this.watchSetCache ??= this.all<WatchSet>("/watch-sets", {})
    return this.watchSetCache
  }

  /** The watched laws of a watch set, fetched once per script. */
  documents(watchSetId: string): Promise<Document[]> {
    let documents = this.documentCache.get(watchSetId)
    if (!documents) {
      documents = this.get<Document[]>(`/watch-sets/${watchSetId}/documents`)
      documents.catch(() => this.documentCache.delete(watchSetId))
      this.documentCache.set(watchSetId, documents)
    }
    return documents
  }

  /** Where the user watches a law or a section; empty when nothing matches. */
  async watched(query: WatchedQuery = {}): Promise<Watched[]> {
    const law = query.law ? fold(query.law) : undefined
    const section = query.section ? sectionKey(query.section) : undefined
    const sets = await this.watchSets()
    const lists = await Promise.all(sets.map((set) => this.documents(set.id)))
    return sets.flatMap((set, index) =>
      lists[index]
        .filter((document) => !law || matchesLaw(law, document.citation, document.title))
        .map((document) => ({
          watchSetId: set.id,
          watchSetName: set.name,
          citation: document.citation,
          title: document.title,
          wholeDocument: document.wholeDocument,
          sections: document.sections.flatMap((item) => (item.label ? [item.label] : [])),
        }))
        .filter(
          (document) =>
            !section || document.wholeDocument || document.sections.some((label) => sectionKey(label) === section),
        ),
    )
  }

  /** Every alert in range, all pages; the filters run here, inclusive at both ends. */
  async alerts(query: AlertQuery = {}): Promise<Alert[]> {
    const raw = await this.all<Record<string, unknown>>("/alerts", {
      watchSetId: query.watchSet,
      acknowledged: query.unacknowledged ? false : undefined,
    })
    const titles = await this.titles(new Set(raw.map((alert) => alert.watchSetId as string)))
    const law = query.law ? fold(query.law) : undefined
    const section = query.section ? sectionKey(query.section) : undefined
    const effective = range(query.effectiveFrom, query.effectiveTo)
    const detected = range(query.detectedFrom, query.detectedTo)
    const alerts = raw
      .map((alert) => this.alertOf(alert, titles))
      .filter((alert) => !law || matchesLaw(law, alert.citation, alert.lawTitle))
      .filter((alert) => !section || (alert.label !== null && sectionKey(alert.label) === section))
      .filter((alert) => !query.types || query.types.includes(alert.type))
      .filter((alert) => inRange(alert.effectiveFrom, effective))
      .filter((alert) => inRange(pragueDate(alert.detectedAt), detected))
      .sort((left, right) => (left.effectiveFrom ?? "9999").localeCompare(right.effectiveFrom ?? "9999"))
    cite(alerts.map(alertSource))
    return alerts
  }

  async alert(id: string): Promise<Alert> {
    const raw = await this.get<Record<string, unknown>>(`/alerts/${id}`)
    const alert = this.alertOf(raw, await this.titles(new Set([raw.watchSetId as string])))
    cite([alertSource(alert)])
    return alert
  }

  private alertOf(raw: Record<string, unknown>, titles: Map<string, string>): Alert {
    const alert = strip(raw) as Alert
    alert.lawTitle = titles.get(alert.documentId) ?? null
    alert.turbolealUrl = alertPageUrl(this.appUrl, alert.id)
    return alert
  }

  /** Law names by CODEXIS document id; a failed lookup only leaves the name out. */
  private async titles(watchSets: Set<string>): Promise<Map<string, string>> {
    const lists = await Promise.all(
      [...watchSets].filter(Boolean).map((id) => this.documents(id).catch(() => [] as Document[])),
    )
    return new Map(lists.flat().map((document) => [document.documentId, document.title]))
  }

  private async all<T>(path: string, query: Query): Promise<T[]> {
    const items: T[] = []
    let after: string | undefined
    do {
      const page = await this.get<Page<T>>(path, { ...query, after, limit: PAGE_LIMIT })
      items.push(...page.items)
      after = page.nextCursor ?? undefined
    } while (after)
    return items
  }

  private async get<T>(path: string, query: Query = {}): Promise<T> {
    if (!this.key) throw new TurbolealError(`Turboleal není připojený. ${UNAUTHORIZED_HINT}`, 401)
    const url = new URL(`${this.appUrl}/api/v1${path}`)
    for (const [name, value] of Object.entries(query)) {
      if (value !== undefined) url.searchParams.set(name, String(value))
    }
    const key = this.key.replace(/^bearer /i, "")
    let response: Response
    try {
      response = await this.fetcher(url, {
        headers: { Authorization: `Bearer ${key}`, Accept: "application/json" },
        signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
      })
    } catch (error) {
      throw new TurbolealError(`Turboleal není dostupný: ${(error as Error).message}`, 0)
    }
    if (!response.ok) throw await problem(response)
    return (await response.json()) as T
  }
}

async function problem(response: Response): Promise<TurbolealError> {
  let detail = ""
  try {
    const body = (await response.json()) as { title?: string; detail?: string }
    detail = body.detail ?? body.title ?? ""
  } catch {}
  const hint = response.status === 401 ? ` ${UNAUTHORIZED_HINT}` : ""
  return new TurbolealError(`HTTP ${response.status}${detail ? `: ${detail}` : ""}${hint}`, response.status)
}

function strip(raw: Record<string, unknown>): Record<string, unknown> {
  const copy = { ...raw }
  for (const field of HIDDEN_FIELDS) delete copy[field]
  return copy
}

function matchesLaw(law: string, citation: string, title: string | null): boolean {
  return fold(citation).includes(law) || (title !== null && fold(title).includes(law))
}

type Range = { from?: string; to?: string }

function range(from?: DateLike, to?: DateLike): Range {
  return { from: from ? isoDate(from) : undefined, to: to ? isoDate(to) : undefined }
}

function inRange(date: string | null, { from, to }: Range): boolean {
  if (!from && !to) return true
  if (!date) return false
  return (!from || date >= from) && (!to || date <= to)
}

function alertSource(alert: Alert): SourceEntry {
  return entry(
    alert.turbolealUrl,
    `Upozornění v Turbolealu: ${provisionTitle(alert.lawTitle, alert.citation, alert.label)}`,
    changeSnippet(alert),
  )
}

function changeSnippet(alert: Alert): string {
  const label = typeLabel(alert.type)
  const parts = [`${label.charAt(0).toUpperCase()}${label.slice(1)}.`]
  if (alert.effectiveFrom) parts.push(`Účinnost od ${czechDate(alert.effectiveFrom)}.`)
  if (alert.amendments.length) {
    parts.push(`Novela č. ${alert.amendments.map((amendment) => amendment.citation).join(", ")}.`)
  }
  return parts.join(" ")
}

/**
 * A compact, user-facing view of an alert: Czech type and dates, whether it is
 * already in force, the Turboleal link, and the diff read paragraph by paragraph.
 */
export function summarize(alert: Alert, today: DateLike = new Date()): Summary {
  return {
    type: typeLabel(alert.type),
    law: alert.lawTitle?.trim() || null,
    citation: alert.citation,
    section: alert.label ?? null,
    effectiveFrom: czechDate(alert.effectiveFrom),
    inForce: alert.effectiveFrom ? alert.effectiveFrom <= isoDate(today) : null,
    detectedAt: czechDate(pragueDate(alert.detectedAt)),
    amendments: alert.amendments.map((amendment) => amendment.citation),
    acknowledged: alert.acknowledgedAt !== null,
    turbolealUrl: alert.turbolealUrl,
    overview: diffOverview(alert.diffMarkdown),
    diffMarkdown: alert.diffMarkdown ?? alert.diff,
  }
}
