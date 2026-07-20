import { apiGet, CliError, type FetchLike } from "../http"
import { EXIT, getApiKey, getBaseUrl, writeApiKey, deleteApiKey } from "../config"

const USAGE =
  "Použití: sid-cli settings set-key|test|status|delete-key  " +
  "(set-key čte klíč ze STDIN: echo KLIC | sid-cli settings set-key)"

async function readStdin(): Promise<string> {
  const chunks: Buffer[] = []
  for await (const c of process.stdin) chunks.push(c as Buffer)
  return Buffer.concat(chunks).toString("utf8")
}

export async function run(
  argv: string[],
  f?: FetchLike,
  stdin: () => Promise<string> = readStdin,
): Promise<unknown> {
  const [sub] = argv

  if (sub === "set-key") {
    // STDIN, nikdy argv — argv je vidět v `ps` i v kontextu modelu.
    const key = (await stdin()).trim()
    if (!key) throw new CliError(`Na STDIN nepřišel žádný klíč. ${USAGE}`, EXIT.USAGE)

    writeApiKey(key)
    try {
      const user = await apiGet("/api/v1/users/me", f)
      // Vracíme jen prefix — plný klíč zná uživatel, do výstupu nepatří.
      return { verified: true, user, start: key.slice(0, 7) }
    } catch (e) {
      // Neplatný klíč neukládáme — jinak by zůstal v .env a každý další
      // příkaz by hlásil 401 bez zjevné příčiny.
      deleteApiKey()
      const err = e as CliError
      throw new CliError(`Klíč nebyl uložen — SID ho odmítl: ${err.message}`, err.exitCode ?? EXIT.ERROR)
    }
  }

  if (sub === "test") {
    const user = await apiGet("/api/v1/users/me", f)
    return { ok: true, user }
  }

  if (sub === "status") {
    const key = getApiKey()
    return { configured: key !== "", baseUrl: getBaseUrl(), ...(key ? { start: key.slice(0, 7) } : {}) }
  }

  if (sub === "delete-key") {
    deleteApiKey()
    return { deleted: true }
  }

  throw new CliError(USAGE, EXIT.USAGE)
}
