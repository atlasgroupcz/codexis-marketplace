import { useCallback } from 'react'
import { useFetch } from './use-fetch'
import { fetchDetail } from '@/lib/api'

export function useDetail(uuid: string) {
  const fetcher = useCallback(() => fetchDetail(uuid), [uuid])
  return useFetch(fetcher, [uuid])
}
