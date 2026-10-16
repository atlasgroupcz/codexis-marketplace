import { describe, expect, it } from "bun:test"
import { type Alert, type Change, type FetchLike, summarize, Turboleal, TurbolealError } from "./client"

const APP = "https://turboleal.example"
const SET_A = "00000000-0000-0000-0000-00000000000a"
const SET_B = "00000000-0000-0000-0000-00000000000b"

function alertBody(id: number, overrides: Record<string, unknown> = {}) {
  return {
    id: `00000000-0000-0000-0000-${String(id).padStart(12, "0")}`,
    watchSetId: SET_A,
    type: "UPCOMING_AMENDMENT",
    documentId: "CR1",
    citation: "586/1992 Sb.",
    sectionId: "00000000-0000-0000-0000-000000000200",
    partId: "paragraf38g",
    label: "§ 38g",
    effectiveFrom: "2027-01-01",
    detectedAt: "2026-10-15T12:00:00Z",
    amendments: [{ documentId: "CR2", citation: "120/2026 Sb." }],
    diff: null,
    diffHtml: "<p>x</p>",
    diffMarkdown: "{--old--}{++new++}",
    codexisUrl: "https://next.codexis.cz/doc/CR1",
    references: [],
    remindedAt: null,
    acknowledgedAt: null,
    ...overrides,
  }
}

const documentsA = [
  { documentId: "CR1", citation: "586/1992 Sb.", title: "Zákon o daních z příjmů", status: "VALID", wholeDocument: false, sections: [] },
  { documentId: "CR3", citation: "262/2006 Sb.", title: "Zákoník práce", status: "VALID", wholeDocument: true, sections: [] },
]

type Route = (url: URL) => unknown

function fakeApi(routes: Record<string, Route>) {
  const calls: URL[] = []
  const fetcher = (async (input: URL | string, init?: RequestInit) => {
    const url = new URL(String(input))
    calls.push(url)
    expect(new Headers(init?.headers).get("authorization")).toBe("Bearer api-key-test")
    const route = routes[url.pathname]
    if (!route) return new Response(JSON.stringify({ title: "Not Found" }), { status: 404 })
    return new Response(JSON.stringify(route(url)), { status: 200 })
  }) as unknown as FetchLike
  return { calls, client: new Turboleal({ apiUrl: `${APP}/`, apiKey: "api-key-test", fetch: fetcher }) }
}

describe("alerts", () => {
  it("follows the cursor, names the law and links each alert to Turboleal only", async () => {
    const { calls, client } = fakeApi({
      "/api/v1/alerts": (url) =>
        url.searchParams.get("after")
          ? { items: [alertBody(2)], nextCursor: null }
          : { items: [alertBody(1)], nextCursor: "00000000-0000-0000-0000-000000000001" },
      [`/api/v1/watch-sets/${SET_A}/documents`]: () => documentsA,
    })

    const alerts = await client.alerts({ unacknowledged: true })

    expect(alerts.map((alert) => alert.id.slice(-1))).toEqual(["1", "2"])
    expect(alerts[0].lawTitle).toBe("Zákon o daních z příjmů")
    expect(alerts[0].turbolealUrl).toBe(
      "https://turboleal.example/alerts?alerts=%7B%22acknowledged%22%3A%22all%22%2C%22alert%22%3A%22QWxlcnQ6MDAwMDAwMDAtMDAwMC0wMDAwLTAwMDAtMDAwMDAwMDAwMDAx%22%7D",
    )
    expect(JSON.stringify(alerts)).not.toContain("codexis")
    expect(calls[0].searchParams.get("acknowledged")).toBe("false")
    expect(calls[0].searchParams.get("limit")).toBe("200")
    expect(calls.filter((url) => url.pathname.endsWith("/documents"))).toHaveLength(1)
  })

  it("filters by law name, section and effective date range on the client", async () => {
    const { client } = fakeApi({
      "/api/v1/alerts": () => ({
        items: [
          alertBody(1),
          alertBody(2, { label: "§ 38ga" }),
          alertBody(3, { effectiveFrom: "2026-07-01", type: "AMENDED" }),
          alertBody(4, { documentId: "CR3", citation: "262/2006 Sb.", label: "§ 35" }),
        ],
        nextCursor: null,
      }),
      [`/api/v1/watch-sets/${SET_A}/documents`]: () => documentsA,
    })

    const byLaw = await client.alerts({ law: "danich z prijmu", section: "38g" })
    const byDate = await client.alerts({ effectiveFrom: "2026-01-01", effectiveTo: new Date(2026, 11, 31) })
    const byType = await client.alerts({ types: ["UPCOMING_AMENDMENT"], law: "262/2006" })

    expect(byLaw.map((alert) => alert.id.slice(-1))).toEqual(["1", "3"])
    expect(byDate.map((alert) => alert.id.slice(-1))).toEqual(["3"])
    expect(byType.map((alert) => alert.id.slice(-1))).toEqual(["4"])
  })

  it("keeps the alert when its law name cannot be looked up", async () => {
    const { client } = fakeApi({ "/api/v1/alerts/abc": () => alertBody(1, { watchSetId: SET_B }) })

    const alert = await client.alert("abc")

    expect(alert.lawTitle).toBeNull()
    expect(alert.diffMarkdown).toBe("{--old--}{++new++}")
  })
})

describe("changes", () => {
  it("passes the range, maps documents to watch sets and drops CODEXIS links", async () => {
    const change = {
      id: "c1",
      documentId: "CR3",
      citation: "262/2006 Sb.",
      title: "Zákoník práce",
      contentItemId: "paragraf35",
      caption: "§ 35",
      type: "CHANGED",
      effectiveFrom: "2026-12-01",
      detectedAt: "2026-10-10T08:00:00Z",
      amendments: [],
      watched: true,
      codexisUrl: "https://next.codexis.cz/doc/CR3",
      diffMarkdown: "a{++b++}",
      diffHtml: "<p>b</p>",
    }
    const { calls, client } = fakeApi({
      "/api/v1/changes": () => ({ items: [change, { ...change, id: "c2", caption: "§ 36", watched: false }], nextCursor: null }),
      "/api/v1/watch-sets": () => ({ items: [{ id: SET_A }, { id: SET_B }], nextCursor: null }),
      [`/api/v1/watch-sets/${SET_A}/documents`]: () => documentsA,
      [`/api/v1/watch-sets/${SET_B}/documents`]: () => [],
    })

    const changes = await client.changes({ from: "2026-10-01", to: "2026-12-31", by: "effective", section: "35" })

    const query = calls.find((url) => url.pathname === "/api/v1/changes")?.searchParams
    expect(query?.get("from")).toBe("2026-10-01")
    expect(query?.get("to")).toBe("2026-12-31")
    expect(query?.get("by")).toBe("effective")
    expect(changes.map((item) => item.id)).toEqual(["c1"])
    expect(changes[0].watchSetIds).toEqual([SET_A])
    expect(changes[0].turbolealUrl).toBe(`${APP}/watch-sets/V2F0Y2hTZXQ6MDAwMDAwMDAtMDAwMC0wMDAwLTAwMDAtMDAwMDAwMDAwMDBh`)
    expect(changes[0].diffHtml).toBe("<p>b</p>")
    expect(JSON.stringify(changes)).not.toContain("codexis")
  })
})

describe("errors", () => {
  it("explains a rejected key in Czech", async () => {
    const fetcher = (async () => new Response("{}", { status: 401 })) as unknown as FetchLike
    const client = new Turboleal({ apiUrl: APP, apiKey: "api-key-x", fetch: fetcher })

    const error = await client.me().catch((caught: unknown) => caught)

    expect(error).toBeInstanceOf(TurbolealError)
    expect((error as TurbolealError).status).toBe(401)
    expect((error as Error).message).toContain("API klíč Turbolealu")
  })

  it("does not call Turboleal without a key", async () => {
    let called = false
    const fetcher = (async () => {
      called = true
      return new Response("{}")
    }) as unknown as FetchLike
    const previous = process.env.TURBOLEAL_API_KEY
    delete process.env.TURBOLEAL_API_KEY
    const previousHome = process.env.CODEXIS_PUBLIC_USER_HOME
    process.env.CODEXIS_PUBLIC_USER_HOME = "/nonexistent"
    try {
      const client = new Turboleal({ apiUrl: APP, fetch: fetcher })
      await expect(client.me()).rejects.toThrow("není připojený")
      expect(called).toBe(false)
    } finally {
      if (previous !== undefined) process.env.TURBOLEAL_API_KEY = previous
      if (previousHome === undefined) delete process.env.CODEXIS_PUBLIC_USER_HOME
      else process.env.CODEXIS_PUBLIC_USER_HOME = previousHome
    }
  })
})

describe("summarize", () => {
  it("turns an alert into Czech words and keeps the Turboleal link and diff", () => {
    const alert = {
      ...alertBody(1),
      lawTitle: "Zákon o daních z příjmů",
      turbolealUrl: `${APP}/alerts?x`,
    } as unknown as Alert

    expect(summarize(alert)).toEqual({
      type: "připravovaná změna, zatím nenabyla účinnosti",
      law: "Zákon o daních z příjmů",
      citation: "586/1992 Sb.",
      section: "§ 38g",
      effectiveFrom: "1. 1. 2027",
      detectedAt: "15. 10. 2026",
      amendments: ["120/2026 Sb."],
      acknowledged: false,
      turbolealUrl: `${APP}/alerts?x`,
      diffMarkdown: "{--old--}{++new++}",
    })
  })

  it("names a change by its caption and tells whether it is watched", () => {
    const change = {
      id: "c1",
      documentId: "CR3",
      citation: "262/2006 Sb.",
      title: "Zákoník práce",
      contentItemId: "paragraf35",
      caption: "§ 35",
      type: "REMOVED",
      effectiveFrom: null,
      detectedAt: "2026-10-10T08:00:00Z",
      amendments: [],
      watched: false,
      diffMarkdown: null,
      diffHtml: null,
      watchSetIds: [],
      turbolealUrl: null,
    } as Change

    expect(summarize(change)).toMatchObject({
      type: "zrušené ustanovení",
      law: "Zákoník práce",
      section: "§ 35",
      effectiveFrom: null,
      watched: false,
    })
  })
})
