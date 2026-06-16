import { useFetch } from './use-fetch'
import { fetchDph } from '@/lib/api'

/** VAT-payer reliability (+ published accounts) for a company, by IČO/DIČ. */
export function useDph(dic: string) {
  return useFetch(() => fetchDph(dic), [dic])
}
