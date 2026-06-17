import { useCallback, useEffect, useState } from 'react'
import type { FolderResponse } from '@/lib/schemas'
import { fetchFolder } from '@/lib/api'

interface UseFolderDetailResult {
  data: FolderResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
}

export function useFolderDetail(root: string): UseFolderDetailResult {
  const [data, setData] = useState<FolderResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const load = useCallback(() => {
    setLoading(true)
    setError(null)
    fetchFolder(root)
      .then(setData)
      .catch((err: unknown) => setError(err instanceof Error ? err : new Error(String(err))))
      .finally(() => setLoading(false))
  }, [root])

  useEffect(() => {
    load()
  }, [load])

  return { data, loading, error, refetch: load }
}
