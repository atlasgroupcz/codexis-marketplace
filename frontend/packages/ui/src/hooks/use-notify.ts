import { useFetch } from './use-fetch'
import type { UseFetchResult } from './use-fetch'

/**
 * Loads the current notification settings for a watchdog app.
 *
 * Thin wrapper over {@link useFetch} shared by the watchdog apps. Each app
 * passes its own `fetchNotify` reader because the request URL and response
 * schema are app-specific, while the fetch/loading/error/refetch lifecycle is
 * identical everywhere.
 */
export function useNotify<T>(fetcher: () => Promise<T>): UseFetchResult<T> {
  return useFetch(fetcher)
}
