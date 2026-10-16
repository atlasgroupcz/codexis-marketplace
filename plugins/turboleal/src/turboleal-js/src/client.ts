import { apiKey as storedApiKey, apiUrl as configuredApiUrl } from "./config"
import {
  alertPageUrl,
  czechDate,
  fold,
  isoDate,
  provisionTitle,
  sectionKey,
  typeLabel,
  watchSetPageUrl,
} from "./format"
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
export type Change = {
  id: string
  documentId: string
  citation: string
  title: string
  contentItemId: string | null
  caption: string | null
  type: "ADDED" | "CHANGED" | "REMOVED"
  effectiveFrom: string | null
  detectedAt: string
  amendments: Amendment[]
  watched: boolean
  diffMarkdown: string | null
  diffHtml: string | null
  watchSetIds: string[]
  turbolealUrl: string | null
}
export type Summary = {
  type: string
  law: string | null
  citation: string
  section: string | null
  effectiveFrom: string | null
  detectedAt: string | null
  amendments: string[]
  acknowledged?: boolean
  watched?: boolean
  turbolealUrl: string | null
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
  types?: string[]
}
export type ChangeQuery = {
  from?: DateLike
  to?: DateLike
  by?: "detected" | "effective"
  watchSet?: string
  law?: string
  section?: string
  watched?: boolean
}

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

  async alerts(query: AlertQuery = {}): Promise<Alert[]> {
    const raw = await this.all<Record<string, unknown>>("/alerts", {
      watchSetId: query.watchSet,
      acknowledged: query.unacknowledged ? false : undefined,
    })
    const titles = await this.titles(new Set(raw.map((alert) => alert.watchSetId as string)))
    const alerts = raw.map((alert) => this.alertOf(alert, titles))
    const law = query.law ? fold(query.law) : undefined
    const section = query.section ? sectionKey(query.section) : undefined
    const from = query.effectiveFrom ? isoDate(query.effectiveFrom) : undefined
    const to = query.effectiveTo ? isoDate(query.effectiveTo) : undefined
    const filtered = alerts
      .filter((alert) => !law || matchesLaw(law, alert.citation, alert.lawTitle))
      .filter((alert) => !section || (alert.label !== null && sectionKey(alert.label) === section))
      .filter((alert) => !query.types || query.types.includes(alert.type))
      .filter((alert) => inRange(alert.effectiveFrom, from, to))
    cite(filtered.map(alertSource))
    return filtered
  }

  async alert(id: string): Promise<Alert> {
    const raw = await this.get<Record<string, unknown>>(`/alerts/${id}`)
    const alert = this.alertOf(raw, await this.titles(new Set([raw.watchSetId as string])))
    cite([alertSource(alert)])
    return alert
  }

  async changes(query: ChangeQuery = {}): Promise<Change[]> {
    const [raw, documents] = await Promise.all([
      this.all<Record<string, unknown>>("/changes", {
        from: query.from ? isoDate(query.from) : undefined,
        to: query.to ? isoDate(query.to) : undefined,
        by: query.by,
        watchSetId: query.watchSet,
      }),
      this.watchedDocuments(query.watchSet),
    ])
    const law = query.law ? fold(query.law) : undefined
    const section = query.section ? sectionKey(query.section) : undefined
    const changes = raw
      .map((change) => this.changeOf(change, documents))
      .filter((change) => !law || matchesLaw(law, change.citation, change.title))
      .filter((change) => !section || (change.caption !== null && sectionKey(change.caption) === section))
      .filter((change) => query.watched === undefined || change.watched === query.watched)
    cite(this.watchSetSources(changes))
    return changes
  }

  private alertOf(raw: Record<string, unknown>, titles: Map<string, string>): Alert {
    const alert = strip(raw) as Alert
    alert.lawTitle = titles.get(alert.documentId) ?? null
    alert.turbolealUrl = alertPageUrl(this.appUrl, alert.id)
    return alert
  }

  private changeOf(raw: Record<string, unknown>, documents: Map<string, string[]>): Change {
    const change = strip(raw, ["codexisUrl"]) as Change
    change.watchSetIds = documents.get(change.documentId) ?? []
    const [watchSet] = change.watchSetIds
    change.turbolealUrl = watchSet ? watchSetPageUrl(this.appUrl, watchSet) : null
    return change
  }

  private watchSetSources(changes: Change[]): SourceEntry[] {
    return changes
      .filter((change) => change.turbolealUrl)
      .map((change) => entry(change.turbolealUrl as string, "Sledovaná legislativa v Turbolealu"))
  }

  /** Document id -> the watch sets that watch it, from every watch set or the one given. */
  private async watchedDocuments(watchSet?: string): Promise<Map<string, string[]>> {
    const ids = watchSet ? [watchSet] : (await this.watchSets()).map((set) => set.id)
    const lists = await Promise.all(ids.map((id) => this.documents(id).catch(() => [] as Document[])))
    const watched = new Map<string, string[]>()
    lists.forEach((documents, index) => {
      for (const document of documents) {
        watched.set(document.documentId, [...(watched.get(document.documentId) ?? []), ids[index]])
      }
    })
    return watched
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

function strip(raw: Record<string, unknown>, fields = HIDDEN_FIELDS): Record<string, unknown> {
  const copy = { ...raw }
  for (const field of fields) delete copy[field]
  return copy
}

function matchesLaw(law: string, citation: string, title: string | null): boolean {
  return fold(citation).includes(law) || (title !== null && fold(title).includes(law))
}

function inRange(date: string | null, from?: string, to?: string): boolean {
  if (!from && !to) return true
  if (!date) return false
  return (!from || date >= from) && (!to || date <= to)
}

function alertSource(alert: Alert): SourceEntry {
  return entry(
    alert.turbolealUrl,
    `Upozornění v Turbolealu: ${provisionTitle(alert.lawTitle, alert.citation, alert.label)}`,
    changeSnippet(alert.type, alert.effectiveFrom, alert.amendments),
  )
}

function changeSnippet(type: string, effectiveFrom: string | null, amendments: Amendment[]): string {
  const label = typeLabel(type)
  const parts = [`${label.charAt(0).toUpperCase()}${label.slice(1)}.`]
  if (effectiveFrom) parts.push(`Účinnost od ${czechDate(effectiveFrom)}.`)
  if (amendments.length) parts.push(`Novela č. ${amendments.map((amendment) => amendment.citation).join(", ")}.`)
  return parts.join(" ")
}

/** A compact, user-facing view of an alert or a change, ready to retell in Czech. */
export function summarize(item: Alert | Change): Summary {
  const isAlert = "sectionId" in item
  const lawTitle = isAlert ? item.lawTitle : item.title
  const summary: Summary = {
    type: typeLabel(item.type),
    law: lawTitle?.trim() || null,
    citation: item.citation,
    section: (isAlert ? item.label : item.caption) ?? null,
    effectiveFrom: czechDate(item.effectiveFrom),
    detectedAt: czechDate(item.detectedAt),
    amendments: item.amendments.map((amendment) => amendment.citation),
    turbolealUrl: item.turbolealUrl,
    diffMarkdown: item.diffMarkdown ?? (isAlert ? item.diff : null),
  }
  if (isAlert) summary.acknowledged = item.acknowledgedAt !== null
  else summary.watched = item.watched
  return summary
}
