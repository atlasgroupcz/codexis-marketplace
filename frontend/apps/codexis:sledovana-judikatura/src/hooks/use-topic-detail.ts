import { useState, useEffect, useCallback } from 'react'
import { fetchDetail } from '@/lib/api'
import type { DetailResponse } from '@/lib/schemas'

export function useTopicDetail(uuid: string) {
  const [data, setData] = useState<DetailResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await fetchDetail(uuid)
      setData(result)
    } catch (e) {
      setError(e instanceof Error ? e : new Error(String(e)))
    } finally {
      setLoading(false)
    }
  }, [uuid])

  useEffect(() => {
    void load()
  }, [load])

  return { data, loading, error, refetch: load }
}
