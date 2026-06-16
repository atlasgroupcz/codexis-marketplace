import { useFetch } from './use-fetch'
import { fetchOverview } from '@/lib/api'

export function useOverview() {
  return useFetch(fetchOverview)
}
