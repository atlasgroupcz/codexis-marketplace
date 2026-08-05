import { parseArgs } from "node:util"
import { apiGet, type FetchLike } from "../http"
import { sendDigest } from "../notify"
import { confirmationsUrl, documentUrl, homeUrl, resolveRole } from "../links"

/**
 * Daily digest for `automations/check-sid.md`.
 *
 * The JSON on stdout is the primary output; `--notify` additionally pushes
 * through the daemon, but only when there is something to report.
 */
export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const { values } = parseArgs({ args: argv, options: { notify: { type: "boolean", default: false } } })

  const [notifications, confirmations] = await Promise.all([
    apiGet("/api/notifications?unreadOnly=true&limit=50", f) as Promise<{ documentId?: string }[]>,
    apiGet("/api/v1/confirmations?state=pending", f) as Promise<{ documentId?: string }[]>,
  ])
  const role = await resolveRole(f)
  const withUrl = <T extends { documentId?: string }>(items: T[]) =>
    items.map((i) => (i.documentId ? { ...i, url: documentUrl(i.documentId, role) } : i))

  const digest = {
    unreadNotifications: notifications.length,
    pendingConfirmations: confirmations.length,
    notifications: withUrl(notifications),
    confirmations: withUrl(confirmations),
    confirmInApp: confirmationsUrl(role),
    notified: false,
  }

  if (values.notify && (digest.unreadNotifications > 0 || digest.pendingConfirmations > 0)) {
    // Pending confirmations are the one thing with an obligation attached,
    // so they win the click. With none, the confirmations page would open
    // empty - send the reader to SID itself instead.
    const link = digest.pendingConfirmations > 0 ? confirmationsUrl(role) : homeUrl()
    const message = `SID: ${digest.unreadNotifications} nových notifikací, ${digest.pendingConfirmations} čekajících potvrzení`
    digest.notified = await sendDigest(message, f, { link, action: "Otevřít v SID" })
  }

  return digest
}
