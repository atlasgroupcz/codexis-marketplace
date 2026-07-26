import { useFetch } from '@workspace/ui/hooks/use-fetch'
import { fetchDetail } from '@/lib/api'

export function useDetail(ico: string) {
  return useFetch(() => fetchDetail(ico), [ico])
}
