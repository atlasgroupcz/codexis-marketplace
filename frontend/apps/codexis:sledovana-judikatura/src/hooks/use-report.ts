import { useState, useEffect, useCallback } from 'react'
import { fetchReport } from '@/lib/api'
import type { ReportResponse } from '@/lib/schemas'

export function useReport(uuid: string, reportId: string) {
  const [data, setData] = useState<ReportResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await fetchReport(uuid, reportId)
      setData(result)
    } catch (e) {
      setError(e instanceof Error ? e : new Error(String(e)))
    } finally {
      setLoading(false)
    }
  }, [uuid, reportId])

  useEffect(() => {
    void load()
  }, [load])

  return { data, loading, error, refetch: load }
}
