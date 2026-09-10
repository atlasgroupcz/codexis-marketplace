import { useCallback, useEffect, useState } from 'react'
import { fetchWatches } from '@/lib/api'
import type { WatchesResponse } from '@/lib/schemas'

interface UseWatchesResult {
  data: WatchesResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
}

export function useWatches(): UseWatchesResult {
  const [data, setData] = useState<WatchesResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const load = useCallback(() => {
    setLoading(true)
    setError(null)
    fetchWatches()
      .then(setData)
      .catch((err: unknown) => setError(err instanceof Error ? err : new Error(String(err))))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  return { data, loading, error, refetch: load }
}
