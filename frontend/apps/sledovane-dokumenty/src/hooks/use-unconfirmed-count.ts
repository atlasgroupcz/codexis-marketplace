import { useFetch } from '@workspace/ui/hooks/use-fetch'
import { fetchFolders, fetchOverview } from '@/lib/api'
import { sumUnconfirmed } from '@/lib/counts'

/**
 * Unconfirmed changes waiting in one watchdog, for the badge on the section rail.
 *
 * A screen calls this for the watchdog it does NOT own; its own half is already
 * in its own data, live, so reading that here would fetch the same endpoint
 * twice and then discard one copy. Every request spawns a CGI process in the
 * user's VM, so a duplicate is not free.
 *
 * Living in the screen that draws the rail also means a detail view fetches
 * nothing, and the number refreshes whenever you cross between watchdogs -
 * switching sections unmounts one home and mounts the other.
 */
export function useUnconfirmedCount(watchdog: 'documents' | 'folders'): number {
  const { data } = useFetch(
    () =>
      watchdog === 'documents'
        ? fetchOverview().then((r) => sumUnconfirmed(r.tracked_documents))
        : fetchFolders().then((r) => sumUnconfirmed(r.folders)),
    [watchdog],
  )
  return data ?? 0
}
