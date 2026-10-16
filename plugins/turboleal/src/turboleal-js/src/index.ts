import { type AlertQuery, type ChangeQuery, type ClientOptions, Turboleal } from "./client"

export { summarize, Turboleal, TurbolealError } from "./client"
export type { Alert, AlertQuery, Change, ChangeQuery, Document, Me, Summary, WatchSet } from "./client"
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
export const changes = (query?: ChangeQuery) => client().changes(query)
