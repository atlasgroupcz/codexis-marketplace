const TYPE_LABELS: Record<string, string> = {
  AMENDED: "změna znění (novela)",
  UPCOMING_AMENDMENT: "připravovaná změna, zatím nenabyla účinnosti",
  ADDED: "nové ustanovení",
  REMOVED: "zrušené ustanovení",
  REPEALED: "zrušený předpis",
}

export function typeLabel(type: string): string {
  return TYPE_LABELS[type] ?? type
}

/** `2027-01-01` or an ISO instant as `1. 1. 2027`; anything else unchanged. */
export function czechDate(value: string | null | undefined): string | null {
  if (!value) return null
  const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(value)
  if (!match) return value
  const [, year, month, day] = match
  return `${Number(day)}. ${Number(month)}. ${year}`
}

/** The GraphQL node id the web app puts into its URLs. */
export function nodeId(typeName: string, id: string): string {
  return Buffer.from(`${typeName}:${id}`).toString("base64url")
}

export function alertPageUrl(appUrl: string, alertId: string): string {
  const state = JSON.stringify({ acknowledged: "all", alert: nodeId("Alert", alertId) })
  return `${appUrl.replace(/\/+$/, "")}/alerts?${new URLSearchParams({ alerts: state })}`
}

/** `Zákon o daních z příjmů (586/1992 Sb.), § 38g`, falling back to the citation. */
export function provisionTitle(law: string | null | undefined, citation: string, label?: string | null): string {
  const name = law?.trim() ? `${law.trim()} (${citation})` : citation
  return label?.trim() ? `${name}, ${label.trim()}` : name
}

/** Lower case without diacritics, for forgiving substring matches. */
export function fold(value: string): string {
  return value.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase()
}

/** `§ 38g`, `38g`, `par. 38g` and `§ 38g - Daňové přiznání` all reduce to `38g`. */
export function sectionKey(value: string): string {
  return fold(value.split(" - ")[0])
    .replace(/§|par\.|paragraf/g, "")
    .replace(/\s+/g, "")
}

const PRAGUE = new Intl.DateTimeFormat("sv-SE", { timeZone: "Europe/Prague", year: "numeric", month: "2-digit", day: "2-digit" })

/** The calendar day in Prague of an instant, e.g. a `detectedAt`. */
export function pragueDate(value: string | Date): string {
  if (typeof value === "string" && /^\d{4}-\d{2}-\d{2}$/.test(value)) return value
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? String(value) : PRAGUE.format(date)
}

/** A `YYYY-MM-DD` string as given, a Date as its day in Prague. */
export function isoDate(value: string | Date): string {
  return value instanceof Date ? pragueDate(value) : value
}
