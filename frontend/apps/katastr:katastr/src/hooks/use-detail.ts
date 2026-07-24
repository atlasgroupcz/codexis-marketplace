import { useFetch } from '@workspace/ui/hooks/use-fetch'
import { fetchDetail } from '@/lib/api'

export function useDetail(cisloRizeni: string) {
  return useFetch(() => fetchDetail(cisloRizeni), [cisloRizeni])
}
