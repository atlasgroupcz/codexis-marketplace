import { useFetch } from './use-fetch'
import { fetchNotify } from '@/lib/api'

export function useNotify() {
  return useFetch(fetchNotify)
}
