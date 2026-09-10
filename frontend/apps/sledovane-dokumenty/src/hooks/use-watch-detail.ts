import { useCallback, useEffect, useState } from 'react'
import { fetchWatch } from '@/lib/api'
import type { WatchResponse } from '@/lib/schemas'

interface UseWatchDetailResult {
  data: WatchResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
}

export function useWatchDetail(watch: string): UseWatchDetailResult {
  const [data, setData] = useState<WatchResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const load = useCallback(() => {
    setLoading(true)
    setError(null)
    fetchWatch(watch)
      .then(setData)
      .catch((err: unknown) => setError(err instanceof Error ? err : new Error(String(err))))
      .finally(() => setLoading(false))
  }, [watch])

  useEffect(() => {
    load()
  }, [load])

  return { data, loading, error, refetch: load }
}
