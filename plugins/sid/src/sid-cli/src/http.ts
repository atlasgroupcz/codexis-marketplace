import { EXIT, getApiKey, getBaseUrl } from "./config"

export class CliError extends Error {
  constructor(
    message: string,
    public readonly exitCode: number,
  ) {
    super(message)
  }
}

export type FetchLike = typeof fetch

const NO_KEY_MSG =
  "API klíč není nastaven. Vygenerujte si ho v SID (Profil → API klíče, " +
  "/profile/api-klice) a vložte v nastavení pluginu SID v aplikaci Codexis."

function statusToError(status: number, bodyText: string): CliError {
  // Tělo ořežeme a NIKDY nepřikládáme hlavičky — klíč nesmí uniknout do chyb.
  const detail = bodyText.slice(0, 300)
  if (status === 401) {
    return new CliError(
      `SID odmítl API klíč (401). Klíč vypršel nebo byl zrušen — vydejte nový na /profile/api-klice. ${detail}`,
      EXIT.AUTH,
    )
  }
  if (status === 403) return new CliError(`Přístup odepřen (403). ${detail}`, EXIT.AUTH)
  if (status === 404) return new CliError(`Nenalezeno (404). ${detail}`, EXIT.NOT_FOUND)
  if (status === 429) {
    return new CliError(`Rate limit SID (429) — zpomalte; search je 60 požadavků/min. ${detail}`, EXIT.RATE_LIMIT)
  }
  return new CliError(`SID vrátil HTTP ${status}. ${detail}`, EXIT.ERROR)
}

/**
 * Strop na jeden požadavek. Bez něj zaseknuté spojení zablokuje denní
 * automatizaci navždy — referenční pluginy proto mají timeout na každém
 * volání (katastr/znamky: 10–30 s).
 */
const REQUEST_TIMEOUT_MS = 30_000

async function request(path: string, init: RequestInit, f: FetchLike, keyOverride?: string): Promise<Response> {
  // `keyOverride` umožní ověřit kandidátský klíč, aniž by se zapsal do
  // .env — viz `settings set-key`.
  const key = keyOverride ?? getApiKey()
  if (!key) throw new CliError(NO_KEY_MSG, EXIT.AUTH)
  const res = await f(`${getBaseUrl()}${path}`, {
    ...init,
    signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    headers: { ...(init.headers ?? {}), "x-api-key": key },
  }).catch((e: unknown) => {
    const name = e instanceof Error ? e.name : ""
    if (name === "TimeoutError" || name === "AbortError") {
      throw new CliError(
        `Požadavek na SID vypršel (${REQUEST_TIMEOUT_MS / 1000} s) — server neodpovídá.`,
        EXIT.ERROR,
      )
    }
    throw new CliError(`SID není dostupný (${e instanceof Error ? e.message : String(e)}).`, EXIT.ERROR)
  })
  if (!res.ok) throw statusToError(res.status, await res.text().catch(() => ""))
  return res
}

/**
 * Odpověď 2xx, která není JSON, znamená, že jsme nemluvili se SID API —
 * typicky proxy/captive portal/login stránka na špatné base URL. Bez
 * tohohle obalu by prosákla syrová hláška "Failed to parse JSON".
 */
async function parseJson(res: Response): Promise<unknown> {
  const text = await res.text()
  try {
    return JSON.parse(text)
  } catch {
    throw new CliError(
      `Odpověď ze SID není JSON — zkontrolujte adresu (${getBaseUrl()}); ` +
        `server vrátil: ${text.slice(0, 120).replace(/\s+/g, " ").trim()}`,
      EXIT.ERROR,
    )
  }
}

export async function apiGet(path: string, f: FetchLike = fetch, keyOverride?: string): Promise<unknown> {
  return parseJson(await request(path, {}, f, keyOverride))
}

export async function apiPatch(path: string, body: unknown, f: FetchLike = fetch): Promise<unknown> {
  return parseJson(
    await request(
      path,
      { method: "PATCH", body: JSON.stringify(body), headers: { "content-type": "application/json" } },
      f,
    ),
  )
}

/** Pro download — vrací Response, tělo si vystreamuje volající. */
export async function apiRaw(path: string, f: FetchLike = fetch): Promise<Response> {
  return request(path, {}, f)
}
