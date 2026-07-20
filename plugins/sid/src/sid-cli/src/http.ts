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

async function request(path: string, init: RequestInit, f: FetchLike): Promise<Response> {
  const key = getApiKey()
  if (!key) throw new CliError(NO_KEY_MSG, EXIT.AUTH)
  const res = await f(`${getBaseUrl()}${path}`, {
    ...init,
    headers: { ...(init.headers ?? {}), "x-api-key": key },
  }).catch((e: unknown) => {
    throw new CliError(`SID není dostupný (${e instanceof Error ? e.message : String(e)}).`, EXIT.ERROR)
  })
  if (!res.ok) throw statusToError(res.status, await res.text().catch(() => ""))
  return res
}

export async function apiGet(path: string, f: FetchLike = fetch): Promise<unknown> {
  return (await request(path, {}, f)).json()
}

export async function apiPatch(path: string, body: unknown, f: FetchLike = fetch): Promise<unknown> {
  return (
    await request(
      path,
      { method: "PATCH", body: JSON.stringify(body), headers: { "content-type": "application/json" } },
      f,
    )
  ).json()
}

/** Pro download — vrací Response, tělo si vystreamuje volající. */
export async function apiRaw(path: string, f: FetchLike = fetch): Promise<Response> {
  return request(path, {}, f)
}
