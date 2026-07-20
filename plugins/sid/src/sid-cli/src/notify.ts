import { readFileSync } from "node:fs"
import { join } from "node:path"
import { randomUUID } from "node:crypto"
import { userHome } from "./config"
import { getNotifySettings } from "./notify-settings"
import type { FetchLike } from "./http"

/**
 * Notifikace zpět uživateli přes daemon plugin API.
 *
 * Auth = VM token z ~/.cdx/env/secret — NE SID API klíč a NE
 * CODEXIS_USER_API_TOKEN (ten gateway odmítne 403; viz `_daemon.py`
 * v pluginu codexis).
 *
 * Best-effort: jakékoli selhání → false. Digest na stdout je primárním
 * výstupem automatizace, notifikace jen bonus — výpadek daemona nesmí
 * shodit naplánovaný běh.
 *
 * Protokol převzat z `plugins/codexis/lib/sledovane_dokumenty_core/notify.py`:
 * e-mail je multipart/form-data s částí `payload` (NE prostý JSON),
 * bell je GraphQL mutace `createNotification`.
 */

const EMAIL_ENDPOINT = "/rest/v1/plugin/email/send"
const GRAPHQL_PATH = "/graphql"

/** CODEXIS_PUBLIC_DAEMON_URL je GraphQL URL — REST base vznikne odříznutím. */
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
  recipients: string[] = [],
): Promise<boolean> {
  // Bez `to` doručí daemon přihlášenému uživateli; pole se přikládá jen
  // když si uživatel nastavil adresáty navíc (vzor katastr/znamky).
  const payload = JSON.stringify(
    recipients.length > 0 ? { subject, bodyText, to: recipients } : { subject, bodyText },
  )
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
      headers: { authorization: auth, "content-type": `multipart/form-data; boundary=${boundary}` },
      body,
    })
    return res.ok
  } catch {
    return false
  }
}

async function sendBell(message: string, auth: string, daemonUrl: string, f: FetchLike): Promise<boolean> {
  const query = "mutation Create($input: CreateNotificationInput!) { createNotification(input: $input) { id } }"
  try {
    const res = await f(graphqlUrl(daemonUrl), {
      method: "POST",
      headers: { authorization: auth, "content-type": "application/json; charset=utf-8" },
      body: JSON.stringify({ query, variables: { input: { message, type: "sid-update" } } }),
    })
    return res.ok
  } catch {
    return false
  }
}

/**
 * Pošle digest kanály povolenými v nastavení (`notify-settings.ts`).
 *
 * Vrací true, když se doručilo aspoň jedním kanálem — e-mail je ve
 * výchozím stavu vypnutý, takže vázat výsledek jen na něj by hlásilo
 * neúspěch i při funkční bell notifikaci.
 */
export async function sendDigest(subject: string, bodyText: string, f: FetchLike = fetch): Promise<boolean> {
  const daemonUrl = process.env.CODEXIS_PUBLIC_DAEMON_URL
  const auth = daemonAuth()
  if (!daemonUrl || !auth) return false

  const { email, inApp, recipients } = getNotifySettings()
  if (!email && !inApp) return false

  const results = await Promise.all([
    email ? sendEmail(subject, bodyText, auth, daemonUrl, f, recipients) : Promise.resolve(false),
    inApp ? sendBell(subject, auth, daemonUrl, f) : Promise.resolve(false),
  ])
  return results.some(Boolean)
}
