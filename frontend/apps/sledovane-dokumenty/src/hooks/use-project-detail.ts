import { useCallback, useEffect, useState } from 'react'
import { fetchProject } from '@/lib/api'
import type { ProjectResponse } from '@/lib/schemas'

interface UseProjectDetailResult {
  data: ProjectResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
}

export function useProjectDetail(project: string): UseProjectDetailResult {
  const [data, setData] = useState<ProjectResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  const load = useCallback(() => {
    setLoading(true)
    setError(null)
    fetchProject(project)
      .then(setData)
      .catch((err: unknown) => setError(err instanceof Error ? err : new Error(String(err))))
      .finally(() => setLoading(false))
  }, [project])

  useEffect(() => {
    load()
  }, [load])

  return { data, loading, error, refetch: load }
}
