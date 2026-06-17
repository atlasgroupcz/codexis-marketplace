import type { Rizeni } from '@/lib/schemas'

const EISIR_BASE = 'https://eisir.justice.cz/insolvencni-rejstrik'

/** Pull the numeric ISIR record id out of a legacy url_detail like
 *  `…/evidence_upadcu_detail.do?id=10009596`. Returns null when absent or non-numeric. */
function isirRecordId(urlDetail: string): string | null {
  if (!urlDetail) return null
  try {
    const id = new URL(urlDetail).searchParams.get('id')
    return id && /^\d+$/.test(id) ? id : null
  } catch {
    return null
  }
}

/** Direct eISIR deep-link to a single proceeding. The record id eISIR uses at
 *  `…/insolvencni-rejstrik/r/{id}` is the same numeric id carried in the legacy
 *  `url_detail`, so we just rebuild the URL. Falls back to the subject-level URL
 *  (search by IČO / register root) when the proceeding carries no record id. */
export function eisirProceedingUrl(rizeni: Rizeni, fallback: string): string {
  const id = isirRecordId(rizeni.url_detail)
  return id ? `${EISIR_BASE}/r/${id}` : fallback
}
