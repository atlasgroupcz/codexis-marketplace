import { useCallback, useEffect, useState } from 'react'

export interface UseFetchResult<T> {
  data: T | null
  loading: boolean
  error: Error | null
  refetch: () => void
}

/**
 * Generic data-fetching hook. Re-runs whenever any value in `deps` changes.
 *
 * Returns:
 * - data: latest fetched value (null until first response)
 * - loading: true while a fetch is in flight
 * - error: last error, cleared on every new fetch
 * - refetch: trigger a manual reload
 */
export function useFetch<T>(
  fetcher: () => Promise<T>,
  deps: ReadonlyArray<unknown> = [],
): UseFetchResult<T> {
  const [data, setData] = useState<T | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const run = useCallback(() => {
    setLoading(true)
    setError(null)
    fetcher()
      .then((result) => setData(result))
      .catch((e: unknown) => setError(e instanceof Error ? e : new Error(String(e))))
      .finally(() => setLoading(false))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps)

  useEffect(() => {
    run()
  }, [run])

  return { data, loading, error, refetch: run }
}
