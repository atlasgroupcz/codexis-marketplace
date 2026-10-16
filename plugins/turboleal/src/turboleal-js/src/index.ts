import { type AlertQuery, type ClientOptions, Turboleal, type WatchedQuery } from "./client"

export { summarize, Turboleal, TurbolealError } from "./client"
export type { Alert, AlertQuery, Document, Me, Summary, Watched, WatchedQuery, WatchSet } from "./client"
export { diffOverview } from "./diff"
export type { DiffItem, DiffOverview } from "./diff"
export { publishSources } from "./sources"

let shared: Turboleal | undefined

/** The client the module functions use; `configure` replaces it, e.g. in tests. */
export function client(): Turboleal {
  shared ??= new Turboleal()
  return shared
}

export function configure(options: ClientOptions = {}): Turboleal {
  shared = new Turboleal(options)
  return shared
}

export const me = () => client().me()
export const watchSets = () => client().watchSets()
export const documents = (watchSetId: string) => client().documents(watchSetId)
export const alerts = (query?: AlertQuery) => client().alerts(query)
export const alert = (id: string) => client().alert(id)
export const watched = (query?: WatchedQuery) => client().watched(query)
