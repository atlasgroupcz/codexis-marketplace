export type ProceedingType = 'vklad' | 'zaznam' | 'poznamka' | 'rizeni'

/** Proceeding kind from the case-number prefix (V- / Z- / P-). */
export function proceedingType(cisloRizeni: string): ProceedingType {
  const p = cisloRizeni.trim().charAt(0).toUpperCase()
  if (p === 'V') return 'vklad'
  if (p === 'Z') return 'zaznam'
  if (p === 'P') return 'poznamka'
  return 'rizeni'
}
