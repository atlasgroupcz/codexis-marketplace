import { useCallback, useEffect, useState } from 'react'
import { fetchFolders } from '@/lib/api'
import type { FoldersResponse } from '@/lib/schemas'

interface UseFoldersResult {
  data: FoldersResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
}

export function useFolders(): UseFoldersResult {
  const [data, setData] = useState<FoldersResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const load = useCallback(() => {
    setLoading(true)
    setError(null)
    fetchFolders()
      .then(setData)
      .catch((err: unknown) => setError(err instanceof Error ? err : new Error(String(err))))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  return { data, loading, error, refetch: load }
}
