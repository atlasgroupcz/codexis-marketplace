/**
 * CGI backend komponenty „SID — API klíč".
 *
 * Protokol převzat z `plugins/znamky/components/znamky/znamky.py` a
 * `cdx-daemon/.../plugin/service/runtime/CgiProtocol.java`:
 *   vstup  — env REQUEST_METHOD / CONTENT_LENGTH / QUERY_STRING, tělo na stdin
 *   výstup — CGI hlavičky, prázdný řádek (CRLF CRLF), pak tělo
 *
 * Daemon servíruje existující soubory v adresáři komponenty staticky
 * (index.html), takže sem doputuje jen požadavek na cestu, která souborem
 * není — frontend proto POSTuje na `api`.
 *
 * Klíč se odsud NIKDY nevrací zpět do UI; jen prefix (`start`).
 */
import { apiGet, type CliError, type FetchLike } from "./http"
import { EXIT, getApiKey, getBaseUrl, writeApiKey, deleteApiKey } from "./config"

type Body = { action?: string; api_key?: string }

function status(): { ok: true; configured: boolean; baseUrl: string; start?: string } {
  const key = getApiKey()
  return { ok: true, configured: key !== "", baseUrl: getBaseUrl(), ...(key ? { start: key.slice(0, 7) } : {}) }
}

export async function handle(body: Body, f: FetchLike = fetch): Promise<unknown> {
  switch (body.action) {
    case "get_status":
      return status()

    case "save_config": {
      const key = (body.api_key ?? "").trim()
      if (!key) return { ok: false, error: "Vložte API klíč." }
      // Ověřit PŘED zápisem — dřív se klíč zapsal a při jakékoli chybě
      // (i výpadku sítě) smazal, takže uživatel přišel i o dosud funkční
      // klíč. `apiGet` bere kandidáta explicitně, .env se nedotýkáme.
      try {
        await apiGet("/api/v1/users/me", f, key)
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
      return { ok: true, verified: true, start: key.slice(0, 7) }
    }

    case "delete_config":
      deleteApiKey()
      return { ok: true }

    default:
      return { ok: false, error: `Neznámá akce '${body.action ?? ""}'.` }
  }
}

/** Sestaví CGI odpověď (hlavičky + prázdný řádek + JSON tělo). */
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
 * Obslouží jeden CGI požadavek a vrátí kompletní odpověď jako řetězec.
 *
 * Nezapisuje na stdout ani nečte proces globálně mimo definovaný vstup —
 * spuštění má na starosti `cgi-main.ts`, aby tenhle modul šel importovat
 * v testech bez vedlejších efektů.
 */
export async function serve(f: FetchLike = fetch): Promise<string> {
  const method = (process.env.REQUEST_METHOD ?? "GET").toUpperCase()
  try {
    // GET = stav konfigurace pro počáteční vykreslení panelu.
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
