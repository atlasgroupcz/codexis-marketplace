import { parseArgs } from "node:util"
import { apiGet, type FetchLike } from "../http"
import { sendDigest } from "../notify"

/**
 * Denní digest pro automatizaci (`automations/check-sid.md`).
 *
 * Stdout JSON je primární výstup; `--notify` navíc pošle e-mail a bell
 * přes daemon, ale jen když je co hlásit.
 */
export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const { values } = parseArgs({ args: argv, options: { notify: { type: "boolean", default: false } } })

  const [notifications, confirmations] = await Promise.all([
    apiGet("/api/notifications?unreadOnly=true&limit=50", f) as Promise<unknown[]>,
    apiGet("/api/v1/confirmations?state=pending", f) as Promise<unknown[]>,
  ])

  const digest = {
    unreadNotifications: notifications.length,
    pendingConfirmations: confirmations.length,
    notifications,
    confirmations,
    notified: false,
  }

  if (values.notify && (digest.unreadNotifications > 0 || digest.pendingConfirmations > 0)) {
    const subject = `SID: ${digest.unreadNotifications} nových notifikací, ${digest.pendingConfirmations} čekajících potvrzení`
    const lines = [
      `Nepřečtené notifikace: ${digest.unreadNotifications}`,
      `Čekající potvrzení: ${digest.pendingConfirmations}`,
      "",
      "Detaily najdete v SID.",
    ]
    digest.notified = await sendDigest(subject, lines.join("\n"), f)
  }

  return digest
}
