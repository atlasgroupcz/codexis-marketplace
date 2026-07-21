import type { Rizeni } from '@/lib/schemas'

/** Deep-link to a single insolvency proceeding.
 *
 *  ISIR's SOAP service already returns a working per-proceeding detail URL
 *  (`urlDetailRizeni`, e.g. `…/evidence_upadcu_detail.do?id=<GUID>`) that
 *  resolves to the exact řízení on isir.justice.cz - so use it verbatim.
 *
 *  We do NOT rebuild an eISIR (`eisir.justice.cz/…/r/{id}`) URL: the id ISIR
 *  carries is a legacy hex GUID, not the record id the new eISIR SPA expects,
 *  so such a URL only lands on the base search page. eISIR is also still in
 *  "ověřovací provoz" (beta) with no public per-proceeding deep-link.
 *
 *  Falls back to the subject-level URL when a proceeding carries no detail URL. */
export function isirProceedingUrl(rizeni: Rizeni, fallback: string): string {
  return rizeni.url_detail || fallback
}
