import { useCallback, useEffect, useState } from 'react'
import { fetchProjects } from '@/lib/api'
import type { ProjectsResponse } from '@/lib/schemas'

interface UseProjectsResult {
  data: ProjectsResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
}

export function useProjects(): UseProjectsResult {
  const [data, setData] = useState<ProjectsResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const load = useCallback(() => {
    setLoading(true)
    setError(null)
    fetchProjects()
      .then(setData)
      .catch((err: unknown) => setError(err instanceof Error ? err : new Error(String(err))))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  return { data, loading, error, refetch: load }
}
