import { useFetch } from '@workspace/ui/hooks/use-fetch'
import { fetchOverview } from '@/lib/api'

export function useOverview() {
  return useFetch(fetchOverview)
}
