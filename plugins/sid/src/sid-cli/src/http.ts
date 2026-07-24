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
  "/api-klice) a vložte v nastavení pluginu SID v aplikaci Codexis."

function statusToError(status: number, bodyText: string): CliError {
  // Truncate the body and never attach headers — the key must not leak into errors.
  const detail = bodyText.slice(0, 300)
  if (status === 401) {
    return new CliError(
      `SID odmítl API klíč (401). Klíč vypršel nebo byl zrušen — vydejte nový na /api-klice. ${detail}`,
      EXIT.AUTH,
    )
  }
  // NOT an expired key — that is 401. SID also answers an id it doesn't know
  // with 403 instead of 404, so it can't be used to probe which ids exist.
  // The usual trigger is a documentId handed to `download`, which wants the
  // fileId from `docs get`, so the id comes first in the advice.
  if (status === 403) {
    return new CliError(
      `Přístup odepřen (403). Nejdřív ověřte id: SID vrací 403 i na neznámé id. ` +
        `Pro download je potřeba fileId z 'sid-cli docs get <documentId>', ne samotné documentId. ` +
        `Pokud id sedí, k dokumentu nemáte přístup. ${detail}`,
      EXIT.FORBIDDEN,
    )
  }
  if (status === 404) return new CliError(`Nenalezeno (404). ${detail}`, EXIT.NOT_FOUND)
  if (status === 429) {
    return new CliError(`Rate limit SID (429) — zpomalte; search je 60 požadavků/min. ${detail}`, EXIT.RATE_LIMIT)
  }
  return new CliError(`SID vrátil HTTP ${status}. ${detail}`, EXIT.ERROR)
}

/** Without a ceiling a stalled connection would block the daily automation forever. */
export const REQUEST_TIMEOUT_MS = 30_000

/**
 * Downloads need their own ceiling: `AbortSignal.timeout` also aborts body
 * reads, so the request timeout would cut off any larger document on a slow
 * link.
 */
export const DOWNLOAD_TIMEOUT_MS = 15 * 60_000

/** Maps a fetch rejection (network down / abort) to a typed CliError. */
function networkError(e: unknown, timeoutMs: number): never {
  const name = e instanceof Error ? e.name : ""
  if (name === "TimeoutError" || name === "AbortError") {
    throw new CliError(`Požadavek na SID vypršel (${timeoutMs / 1000} s) — server neodpovídá.`, EXIT.ERROR)
  }
  throw new CliError(`SID není dostupný (${e instanceof Error ? e.message : String(e)}).`, EXIT.ERROR)
}

async function request(
  path: string,
  init: RequestInit,
  f: FetchLike,
  keyOverride?: string,
  timeoutMs: number = REQUEST_TIMEOUT_MS,
): Promise<Response> {
  // `keyOverride` verifies a candidate key without persisting it (`settings set-key`).
  const key = keyOverride ?? getApiKey()
  if (!key) throw new CliError(NO_KEY_MSG, EXIT.AUTH)
  const res = await f(`${getBaseUrl()}${path}`, {
    ...init,
    signal: AbortSignal.timeout(timeoutMs),
    headers: { ...(init.headers ?? {}), "x-api-key": key },
  }).catch((e: unknown) => networkError(e, timeoutMs))
  if (!res.ok) throw statusToError(res.status, await res.text().catch(() => ""))
  return res
}

/**
 * A 2xx that isn't JSON means this wasn't the SID API at all — typically a
 * proxy, captive portal or login page behind a wrong base URL.
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

/**
 * For downloads — returns the Response so the caller can stream the body.
 *
 * SID's download route 302s to a short-lived presigned URL on the storage
 * host (SeaweedFS/S3), a DIFFERENT origin. `fetch` does not strip a custom
 * `x-api-key` header across origins, so following the redirect automatically
 * would hand the SID key to the storage layer. We follow the redirect
 * ourselves, without the key — the presigned query params are the auth.
 */
export async function apiRaw(
  path: string,
  f: FetchLike = fetch,
  timeoutMs: number = DOWNLOAD_TIMEOUT_MS,
): Promise<Response> {
  const key = getApiKey()
  if (!key) throw new CliError(NO_KEY_MSG, EXIT.AUTH)
  const url = `${getBaseUrl()}${path}`
  const res = await f(url, {
    signal: AbortSignal.timeout(timeoutMs),
    redirect: "manual",
    headers: { "x-api-key": key },
  }).catch((e: unknown) => networkError(e, timeoutMs))

  if (res.status >= 300 && res.status < 400) {
    const location = res.headers.get("location")
    if (!location) throw new CliError(`SID vrátil přesměrování bez cíle (HTTP ${res.status}).`, EXIT.ERROR)
    const target = new URL(location, url).toString()
    const followed = await f(target, {
      signal: AbortSignal.timeout(timeoutMs),
      redirect: "follow",
    }).catch((e: unknown) => networkError(e, timeoutMs))
    if (!followed.ok) throw statusToError(followed.status, await followed.text().catch(() => ""))
    return followed
  }
  if (!res.ok) throw statusToError(res.status, await res.text().catch(() => ""))
  return res
}
