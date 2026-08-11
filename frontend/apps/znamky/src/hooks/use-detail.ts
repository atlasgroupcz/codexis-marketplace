import { useFetch } from '@workspace/ui/hooks/use-fetch'
import { useCallback } from 'react'
import { fetchDetail } from '@/lib/api'

export function useDetail(uuid: string) {
  const fetcher = useCallback(() => fetchDetail(uuid), [uuid])
  return useFetch(fetcher, [uuid])
}
