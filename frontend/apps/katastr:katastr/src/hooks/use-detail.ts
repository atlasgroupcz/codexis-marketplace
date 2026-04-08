import { fetchDetail } from '@/lib/api'
import { useFetch } from './use-fetch'

export function useDetail(uuid: string) {
  return useFetch(() => fetchDetail(uuid), [uuid])
}
