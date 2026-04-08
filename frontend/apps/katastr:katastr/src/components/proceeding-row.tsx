import { useTranslation } from 'react-i18next'
import { Badge } from '@workspace/ui/components/badge'
import { StavBadge } from './stav-badge'
import { UhradaBadge } from './uhrada-badge'
import { formatDateTime } from '@/lib/format'
import type { ProceedingSummary } from '@/lib/schemas'

interface ProceedingRowProps {
  proceeding: ProceedingSummary
  onClick: (uuid: string) => void
}

export function ProceedingRow({ proceeding, onClick }: ProceedingRowProps) {
  const { t } = useTranslation()

  const lastCheckFormatted = proceeding.last_check_at
    ? formatDateTime(proceeding.last_check_at)
    : '---'

  return (
    <tr
      className="hover:bg-muted/30 cursor-pointer border-b transition-colors last:border-b-0 [&>td]:align-middle"
      onClick={() => onClick(proceeding.uuid)}
      data-testid={`proceeding-row-${proceeding.cislo_rizeni}`}
    >
      <td className="px-4 py-3">
        <div className="flex items-center gap-2">
          <span className="font-medium">{proceeding.cislo_rizeni}</span>
          {proceeding.unconfirmed_count > 0 && (
            <Badge variant="destructive">
              {t('proceedings.unconfirmedChangesBadge', {
                count: proceeding.unconfirmed_count,
              })}
            </Badge>
          )}
        </div>
        <div className="text-muted-foreground text-xs">
          {t('proceedings.updated', { date: lastCheckFormatted })}
        </div>
      </td>
      <td className="px-4 py-3 text-sm">
        {proceeding.label || <span className="text-muted-foreground">---</span>}
      </td>
      <td className="px-4 py-3">
        <StavBadge stav={proceeding.stav} />
      </td>
      <td className="px-4 py-3">
        <div className="space-y-0.5">
          {proceeding.provedene_operace.map((op, i) => (
            <div key={i} className="text-xs">
              <span>{op.nazev}</span>
            </div>
          ))}
        </div>
      </td>
      <td className="px-4 py-3">
        <UhradaBadge
          stavUhrady={proceeding.stav_uhrady}
          label={proceeding.stav_uhrady_label}
        />
      </td>
    </tr>
  )
}
