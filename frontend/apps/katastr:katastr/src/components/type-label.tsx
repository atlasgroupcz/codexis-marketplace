import { useTranslation } from 'react-i18next'

type TypeKey = 'vklad' | 'zaznam' | 'poznamka' | 'rizeni'

/** Proceeding kind from the case number prefix (V- / Z- / P-). */
function typeKey(cisloRizeni: string): TypeKey {
  const p = cisloRizeni.trim().charAt(0).toUpperCase()
  if (p === 'V') return 'vklad'
  if (p === 'Z') return 'zaznam'
  if (p === 'P') return 'poznamka'
  return 'rizeni'
}

/** Quiet, low-contrast kind label — secondary to the (coloured) status badge. */
export function TypeLabel({
  cisloRizeni,
  className,
}: {
  cisloRizeni: string
  className?: string
}) {
  const { t } = useTranslation()
  return (
    <span
      className={`text-muted-foreground block text-[11px] font-semibold tracking-wider uppercase ${className ?? ''}`}
      data-testid="type-label"
    >
      {t(`proceedings.type.${typeKey(cisloRizeni)}`)}
    </span>
  )
}
