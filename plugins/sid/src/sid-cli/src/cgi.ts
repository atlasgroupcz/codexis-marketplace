/**
 * CGI backend for the "SID — API key" component.
 *
 * Protocol (see plugins/znamky/components/znamky/znamky.py and the daemon's
 * CgiProtocol.java): input arrives as REQUEST_METHOD / CONTENT_LENGTH /
 * QUERY_STRING env vars plus a body on stdin; output is CGI headers, a blank
 * CRLF line, then the body.
 *
 * The daemon serves existing files in the component directory statically, so
 * only a path that isn't a file reaches this — hence the frontend POSTs to
 * `api`.
 *
 * The key is never returned to the UI, only its prefix.
 */
import { apiGet, type CliError, type FetchLike } from "./http"
import { EXIT, getApiKey, getBaseUrl, writeApiKey, writeEnvValue, deleteApiKey } from "./config"
import {
  getNotifySettings,
  isEmail,
  writeNotifyOverride,
  type NotifySettings,
} from "./notify-settings"

type Body = {
  action?: string
  api_key?: string
  email?: boolean
  inApp?: boolean
  recipients?: unknown[]
}

function status(): {
  ok: true
  configured: boolean
  baseUrl: string
  start?: string
  notify: NotifySettings
} {
  const key = getApiKey()
  return {
    ok: true,
    configured: key !== "",
    baseUrl: getBaseUrl(),
    ...(key ? { start: key.slice(0, 7) } : {}),
    notify: getNotifySettings(),
  }
}

export async function handle(body: Body, f: FetchLike = fetch): Promise<unknown> {
  switch (body.action) {
    case "get_status":
      return status()

    case "save_config": {
      const key = (body.api_key ?? "").trim()
      if (!key) return { ok: false, error: "Vložte API klíč." }
      // Verify before touching .env, so a failed attempt can't destroy a key
      // that still works.
      let me: { role?: unknown }
      try {
        me = (await apiGet("/api/v1/users/me", f, key)) as { role?: unknown }
      } catch (e) {
        const err = e as CliError
        return {
          ok: false,
          error:
            err.exitCode === EXIT.AUTH
              ? "SID klíč odmítl — zkontrolujte, že je zkopírovaný celý a že nevypršel."
              : `Klíč se nepodařilo ověřit (SID neodpovědělo): ${err.message}`,
        }
      }
      writeApiKey(key)
      writeEnvValue("SID_USER_ROLE", me.role === "ADMIN" ? "ADMIN" : "USER")
      return { ok: true, verified: true, start: key.slice(0, 7) }
    }

    case "save_notify_settings": {
      // Only what was actually sent is stored; the file stays a sparse override.
      const patch: Record<string, unknown> = {}
      if (typeof body.email === "boolean") patch.email = body.email
      if (typeof body.inApp === "boolean") patch.inApp = body.inApp
      if (Array.isArray(body.recipients)) {
        // Reject rather than silently drop — otherwise a UI typo would read
        // as "saved" with a recipient missing.
        const invalid = body.recipients.filter(
          (r) => typeof r !== "string" || (r.trim() !== "" && !isEmail(r)),
        )
        if (invalid.length > 0) {
          return { ok: false, error: `Neplatná e-mailová adresa: ${invalid.map(String).join(", ")}` }
        }
        patch.recipients = body.recipients
          .filter((r): r is string => typeof r === "string")
          .map((r) => r.trim())
          .filter(Boolean)
      }
      writeNotifyOverride(patch)
      return { ok: true, notify: getNotifySettings() }
    }

    case "delete_config":
      deleteApiKey()
      return { ok: true }

    default:
      return { ok: false, error: `Neznámá akce '${body.action ?? ""}'.` }
  }
}

export function renderCgi(payload: unknown, cgiStatus?: string): string {
  const head =
    (cgiStatus ? `Status: ${cgiStatus}\r\n` : "") +
    "Content-Type: application/json; charset=utf-8\r\n" +
    "Cache-Control: no-store\r\n"
  return `${head}\r\n${JSON.stringify(payload, null, 2)}`
}

async function readStdin(): Promise<string> {
  const declared = Number(process.env.CONTENT_LENGTH ?? "0")
  if (!Number.isFinite(declared) || declared <= 0) return ""
  const chunks: Buffer[] = []
  for await (const c of process.stdin) chunks.push(c as Buffer)
  return Buffer.concat(chunks).toString("utf8")
}

/**
 * Handles one CGI request and returns the complete response as a string.
 *
 * Writing to stdout is `cgi-main.ts`'s job, so this module can be imported in
 * tests without side effects.
 */
export async function serve(f: FetchLike = fetch): Promise<string> {
  const method = (process.env.REQUEST_METHOD ?? "GET").toUpperCase()
  try {
    // GET returns the configuration state for the panel's initial render.
    if (method !== "POST") return renderCgi(status())

    const raw = await readStdin()
    let body: Body
    try {
      body = raw ? (JSON.parse(raw) as Body) : {}
    } catch {
      return renderCgi({ ok: false, error: "Neplatné JSON tělo." }, "400 Bad Request")
    }
    return renderCgi(await handle(body, f))
  } catch (e) {
    return renderCgi(
      { ok: false, error: `Interní chyba: ${e instanceof Error ? e.message : String(e)}` },
      "500 Internal Server Error",
    )
  }
}
