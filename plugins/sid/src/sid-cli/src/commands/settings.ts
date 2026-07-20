import { apiGet, CliError, type FetchLike } from "../http"
import { EXIT, getApiKey, getBaseUrl, readEnvFile, writeApiKey, writeEnvValue, deleteApiKey } from "../config"

const USAGE =
  "Použití: sid-cli settings set-key|test|status|whoami|delete-key  " +
  "(set-key čte klíč ze STDIN: echo KLIC | sid-cli settings set-key)"

/** Only ADMIN is admin; anything else — including an unknown value — is not. */
function isAdmin(role: unknown): boolean {
  return role === "ADMIN"
}

async function readStdin(): Promise<string> {
  const chunks: Buffer[] = []
  for await (const c of process.stdin) chunks.push(c as Buffer)
  return Buffer.concat(chunks).toString("utf8")
}

/**
 * Verifies a candidate key without persisting it. "Rejected" must only be
 * reported for an actual rejection, never for an unreachable server.
 */
async function verifyKey(key: string, f?: FetchLike): Promise<unknown> {
  try {
    return await apiGet("/api/v1/users/me", f, key)
  } catch (e) {
    const err = e as CliError
    const code = err.exitCode ?? EXIT.ERROR
    const reason =
      code === EXIT.AUTH
        ? `SID klíč odmítl — zkontrolujte, že je zkopírovaný celý a že nevypršel. ${err.message}`
        : `Klíč se nepodařilo ověřit (SID neodpovědělo): ${err.message}`
    throw new CliError(`${reason} Původní nastavení zůstalo beze změny.`, code)
  }
}

export async function run(
  argv: string[],
  f?: FetchLike,
  stdin: () => Promise<string> = readStdin,
): Promise<unknown> {
  const [sub] = argv

  if (sub === "set-key") {
    // STDIN, never argv — argv shows up in `ps` and in the model context.
    const key = (await stdin()).trim()
    if (!key) throw new CliError(`Na STDIN nepřišel žádný klíč. ${USAGE}`, EXIT.USAGE)

    // Verify before touching .env, so a failed attempt can't destroy a key
    // that still works.
    const user = await verifyKey(key, f)
    writeApiKey(key)
    // Cached so links pick the right route without an extra request.
    writeEnvValue("SID_USER_ROLE", (user as { role?: unknown })?.role === "ADMIN" ? "ADMIN" : "USER")
    // Only the prefix goes back out; the user already has the key itself.
    return { verified: true, user, start: key.slice(0, 7) }
  }

  if (sub === "test") {
    const user = await apiGet("/api/v1/users/me", f)
    return { ok: true, user }
  }

  if (sub === "whoami") {
    // The one place role is asked for live: it drives whether admin-scoped
    // actions are worth attempting. USER and ADMIN are SID's only roles.
    const user = (await apiGet("/api/v1/users/me", f)) as {
      id?: string
      email?: string
      name?: string
      role?: unknown
    }
    const admin = isAdmin(user.role)
    return {
      userId: user.id,
      email: user.email,
      name: user.name,
      role: admin ? "ADMIN" : "USER",
      isAdmin: admin,
    }
  }

  if (sub === "status") {
    const key = getApiKey()
    // Role is only known once it has been cached (at set-key), so it is
    // surfaced when present rather than fetched here — status stays offline.
    const cachedRole = readEnvFile().SID_USER_ROLE
    return {
      configured: key !== "",
      baseUrl: getBaseUrl(),
      ...(key ? { start: key.slice(0, 7) } : {}),
      ...(cachedRole ? { role: cachedRole, isAdmin: isAdmin(cachedRole) } : {}),
    }
  }

  if (sub === "delete-key") {
    deleteApiKey()
    return { deleted: true }
  }

  throw new CliError(USAGE, EXIT.USAGE)
}
