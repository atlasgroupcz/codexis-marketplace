import { useCallback, useEffect, useState } from 'react'
import type { OverviewResponse } from '@/lib/schemas'
import { fetchOverview } from '@/lib/api'

interface UseOverviewResult {
  data: OverviewResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
}

export function useOverview(): UseOverviewResult {
  const [data, setData] = useState<OverviewResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const load = useCallback(() => {
    setLoading(true)
    setError(null)
    fetchOverview()
      .then(setData)
      .catch((err: unknown) =>
        setError(err instanceof Error ? err : new Error(String(err))),
      )
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  return { data, loading, error, refetch: load }
}
