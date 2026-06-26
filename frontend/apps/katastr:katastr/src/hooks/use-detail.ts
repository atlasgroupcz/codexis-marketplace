import { useFetch } from './use-fetch'
import { fetchDetail } from '@/lib/api'

export function useDetail(uuid: string) {
  return useFetch(() => fetchDetail(uuid), [uuid])
}
