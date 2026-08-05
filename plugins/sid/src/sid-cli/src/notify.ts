import { readFileSync } from "node:fs"
import { join } from "node:path"
import { randomUUID } from "node:crypto"
import { readEnvFile, userHome, writeEnvValue } from "./config"
import { getNotifySettings } from "./notify-settings"
import type { FetchLike } from "./http"

/**
 * User-facing notifications through the daemon plugin API.
 *
 * Auth is the VM token from ~/.cdx/env/secret — NOT the SID API key and NOT
 * CODEXIS_USER_API_TOKEN, which the gateway rejects with 403.
 *
 * Best-effort throughout: any failure returns false. The digest on stdout is
 * what the automation is for; a daemon outage must not fail the run.
 *
 * Wire format (see plugins/codexis/lib/sledovane_dokumenty_core/notify.py):
 * e-mail is multipart/form-data with a `payload` part, not plain JSON; the
 * bell is the GraphQL `createNotification` mutation.
 */

const EMAIL_ENDPOINT = "/rest/v1/plugin/email/send"
const GRAPHQL_PATH = "/graphql"

/**
 * Ceilings for daemon calls — this is the layer the cron job hangs on.
 * Values follow the reference: 30s for delivery, 10s for the helper query.
 */
const DAEMON_TIMEOUT_MS = 30_000
const ME_QUERY_TIMEOUT_MS = 10_000

/** CODEXIS_PUBLIC_DAEMON_URL is the GraphQL URL; the REST base is it minus /graphql. */
export function restBase(daemonUrl: string): string {
  const base = (daemonUrl || "").replace(/\/+$/, "")
  return base.endsWith(GRAPHQL_PATH) ? base.slice(0, -GRAPHQL_PATH.length).replace(/\/+$/, "") : base
}

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

async function sendEmail(
  subject: string,
  bodyText: string,
  auth: string,
  daemonUrl: string,
  f: FetchLike,
  to: string[] | null,
): Promise<boolean> {
  // Without `to` the daemon delivers to the signed-in user.
  const payload = JSON.stringify(to && to.length > 0 ? { subject, bodyText, to } : { subject, bodyText })
  const boundary = `----sid-${randomUUID().replace(/-/g, "")}`
  const body =
    `--${boundary}\r\n` +
    'Content-Disposition: form-data; name="payload"\r\n' +
    "Content-Type: application/json; charset=utf-8\r\n\r\n" +
    `${payload}\r\n` +
    `--${boundary}--\r\n`
  try {
    const res = await f(`${restBase(daemonUrl)}${EMAIL_ENDPOINT}`, {
      method: "POST",
      signal: AbortSignal.timeout(DAEMON_TIMEOUT_MS),
      headers: { authorization: auth, "content-type": `multipart/form-data; boundary=${boundary}` },
      body,
    })
    return res.ok
  } catch {
    return false
  }
}

/** Signed-in user's e-mail from the daemon, or null. Best-effort. */
async function queryMeEmail(auth: string, daemonUrl: string, f: FetchLike): Promise<string | null> {
  try {
    const res = await f(graphqlUrl(daemonUrl), {
      method: "POST",
      signal: AbortSignal.timeout(ME_QUERY_TIMEOUT_MS),
      headers: { authorization: auth, "content-type": "application/json" },
      body: JSON.stringify({ query: "query { me { email } }" }),
    })
    if (!res.ok) return null
    const data = (await res.json()) as { data?: { me?: { email?: unknown } } }
    const email = data?.data?.me?.email
    return typeof email === "string" && email ? email : null
  } catch {
    return null
  }
}

/**
 * Builds the `to` list.
 *
 * When `to` is present the daemon delivers EXCLUSIVELY to it, so the
 * signed-in user has to be added explicitly — otherwise adding a colleague
 * would silently stop their own mail. With no extras the field is omitted;
 * with extras it is `[me, ...extras]`, deduped. If `me` can't be resolved,
 * send to the extras rather than dropping a real recipient.
 */
async function resolveTo(
  recipients: string[],
  auth: string,
  daemonUrl: string,
  f: FetchLike,
): Promise<string[] | null> {
  // Dedup the extras among themselves (case-insensitively) so a repeated
  // address doesn't send the same digest twice.
  const seen = new Set<string>()
  const extras: string[] = []
  for (const r of recipients) {
    const t = r.trim()
    if (!t) continue
    const k = t.toLowerCase()
    if (seen.has(k)) continue
    seen.add(k)
    extras.push(t)
  }
  if (extras.length === 0) return null
  const me = await queryMeEmail(auth, daemonUrl, f)
  if (!me) return extras
  const meKey = me.trim().toLowerCase()
  return [me, ...extras.filter((e) => e.toLowerCase() !== meKey)]
}

/**
 * Where the bell takes the reader, and what the button says.
 *
 * The notification centre renders `link` as a plain anchor, so an absolute
 * SID URL is fine (and is what we want — the plugin's own panel is a
 * settings screen, not a place to act on a document).
 */
export type BellTarget = { link?: string; action?: string }

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
  if (target.action) input.action = target.action
  try {
    const res = await f(graphqlUrl(daemonUrl), {
      method: "POST",
      signal: AbortSignal.timeout(DAEMON_TIMEOUT_MS),
      headers: { authorization: auth, "content-type": "application/json; charset=utf-8" },
      body: JSON.stringify({ query, variables: { input } }),
    })
    return res.ok
  } catch {
    return false
  }
}

/**
 * Signed-in user's address for the settings panel, or "" when unavailable.
 *
 * Cached in the env file: the panel asks on every open and the address does
 * not change, so without this each render would cost a daemon round trip.
 * A failed lookup caches nothing, so it retries next time.
 */
export async function currentUserEmail(f: FetchLike = fetch): Promise<string> {
  const cached = readEnvFile().SID_USER_EMAIL
  if (cached) return cached

  const daemonUrl = process.env.CODEXIS_PUBLIC_DAEMON_URL
  const auth = daemonAuth()
  if (!daemonUrl || !auth) return ""

  const email = await queryMeEmail(auth, daemonUrl, f)
  if (!email) return ""
  writeEnvValue("SID_USER_EMAIL", email)
  return email
}

/**
 * Sends the digest through whichever channels the settings enable.
 *
 * True means at least one channel delivered — e-mail is off by default, so
 * keying the result to it alone would report failure on a working bell.
 *
 * `target` is the bell's way back into SID. The e-mail body carries its own
 * link inline; the bell has nowhere to put one except this field, and
 * without it the default channel is a count the reader cannot act on.
 */
export async function sendDigest(
  subject: string,
  bodyText: string,
  f: FetchLike = fetch,
  target: BellTarget = {},
): Promise<boolean> {
  const daemonUrl = process.env.CODEXIS_PUBLIC_DAEMON_URL
  const auth = daemonAuth()
  if (!daemonUrl || !auth) return false

  const { email, inApp, recipients } = getNotifySettings()
  if (!email && !inApp) return false

  const to = email ? await resolveTo(recipients, auth, daemonUrl, f) : null
  const results = await Promise.all([
    email ? sendEmail(subject, bodyText, auth, daemonUrl, f, to) : Promise.resolve(false),
    inApp ? sendBell(subject, auth, daemonUrl, f, target) : Promise.resolve(false),
  ])
  return results.some(Boolean)
}
