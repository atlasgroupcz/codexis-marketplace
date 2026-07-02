import { useTranslation } from 'react-i18next'
import { proceedingType } from '@/lib/proceeding'

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
      {t(`proceedings.type.${proceedingType(cisloRizeni)}`)}
    </span>
  )
}
