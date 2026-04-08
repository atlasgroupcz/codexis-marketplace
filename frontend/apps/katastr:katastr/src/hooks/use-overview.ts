import { fetchOverview } from '@/lib/api'
import { useFetch } from './use-fetch'

export function useOverview() {
  return useFetch(fetchOverview)
}
