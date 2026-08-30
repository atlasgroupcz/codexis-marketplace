import { readFileSync } from "node:fs"
import { join } from "node:path"
import { userHome } from "./config"
import { getNotifySettings } from "./notify-settings"
import type { FetchLike } from "./http"

/**
 * User-facing notifications through the daemon plugin API.
 *
 * One channel only: the notification-centre bell. SID sends its own e-mail
 * (per-event templates, a link to the document, per-tenant gating), so a
 * mailer here would duplicate it with a strictly worse message - counts
 * instead of the document that needs attention.
 *
 * Auth is the VM token from ~/.cdx/env/secret - NOT the SID API key and NOT
 * CODEXIS_USER_API_TOKEN, which the gateway rejects with 403.
 *
 * Best-effort throughout: any failure returns false. The digest on stdout is
 * what the automation is for; a daemon outage must not fail the run.
 *
 * Wire format (see plugins/codexis/lib/sledovane_dokumenty_core/notify.py):
 * the bell is the GraphQL `createNotification` mutation.
 */

const GRAPHQL_PATH = "/graphql"

/**
 * Ceiling for daemon calls - this is the layer the cron job hangs on.
 * Follows the reference: 30s for delivery.
 */
const DAEMON_TIMEOUT_MS = 30_000

export function graphqlUrl(daemonUrl: string): string {
  const base = (daemonUrl || "").replace(/\/+$/, "")
  return base.endsWith(GRAPHQL_PATH) ? base : base + GRAPHQL_PATH
}

function daemonAuth(): string | null {
  try {
    let t = readFileSync(join(userHome(), ".cdx", "env", "secret"), "utf8").trim()
    if (!t) return null
    if (t.toLowerCase().startsWith("authorization:")) t = t.slice("authorization:".length).trim()
    // Strip CR/LF: an interior newline in the secret file would otherwise ride
    // along as header injection into every daemon request.
    t = t.replace(/[\r\n]/g, "")
    if (!t) return null
    return t.toLowerCase().startsWith("bearer ") ? t : `Bearer ${t}`
  } catch {
    return null
  }
}

/**
 * Where the bell takes the reader.
 *
 * The notification centre renders `link` as a plain anchor, so an absolute
 * SID URL is fine (and is what we want - the plugin's own panel is a
 * settings screen, not a place to act on a document).
 *
 * There is deliberately no call-to-action field: `CreateNotificationInput`
 * defines `message`, `content`, `link`, `type` and `vars`, and the daemon
 * rejects the whole mutation over an input field it does not know.
 */
export type BellTarget = { link?: string }

async function sendBell(
  message: string,
  auth: string,
  daemonUrl: string,
  f: FetchLike,
  target: BellTarget,
): Promise<boolean> {
  const query = "mutation Create($input: CreateNotificationInput!) { createNotification(input: $input) { id } }"
  // Omit rather than send null: the fields are optional in the daemon's
  // schema and the peer plugins leave them out when there is no target.
  const input: Record<string, string> = { message, type: "sid-update" }
  if (target.link) input.link = target.link
  try {
    const res = await f(graphqlUrl(daemonUrl), {
      method: "POST",
      signal: AbortSignal.timeout(DAEMON_TIMEOUT_MS),
      headers: { authorization: auth, "content-type": "application/json; charset=utf-8" },
      body: JSON.stringify({ query, variables: { input } }),
    })
    if (!res.ok) return false
    // The status alone does not say whether the mutation ran: a rejected one
    // comes back as HTTP 200 with the reason in `errors`, so reading only the
    // status reports every rejection as a delivery. `res.json()` throwing on a
    // body that is not JSON at all - a proxy error page - lands in the catch.
    const payload: unknown = await res.json()
    if (typeof payload !== "object" || payload === null || Array.isArray(payload)) return false
    return !(payload as { errors?: unknown }).errors
  } catch {
    return false
  }
}

/**
 * Puts the digest in the notification centre. False means it did not get
 * there (no daemon, bell switched off, or the call failed) - never throws.
 *
 * `target` is the way back into SID. The bell has nowhere to put a link
 * except this field, and without it the message is a count the reader
 * cannot act on.
 */
export async function sendDigest(
  message: string,
  f: FetchLike = fetch,
  target: BellTarget = {},
): Promise<boolean> {
  const daemonUrl = process.env.CODEXIS_PUBLIC_DAEMON_URL
  const auth = daemonAuth()
  if (!daemonUrl || !auth) return false

  if (!getNotifySettings().inApp) return false

  return sendBell(message, auth, daemonUrl, f, target)
}
