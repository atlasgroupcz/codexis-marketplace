import { useCallback, useEffect, useState } from 'react'
import type { DetailResponse } from '@/lib/schemas'
import { fetchDetail } from '@/lib/api'

interface UseDocumentDetailResult {
  data: DetailResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
}

export function useDocumentDetail(uuid: string): UseDocumentDetailResult {
  const [data, setData] = useState<DetailResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const load = useCallback(() => {
    setLoading(true)
    setError(null)
    fetchDetail(uuid)
      .then(setData)
      .catch((err: unknown) =>
        setError(err instanceof Error ? err : new Error(String(err))),
      )
      .finally(() => setLoading(false))
  }, [uuid])

  useEffect(() => {
    load()
  }, [load])

  return { data, loading, error, refetch: load }
}
