import { useTranslation } from 'react-i18next'
import { StavBadge } from '../stav-badge'
import { UhradaBadge } from '../uhrada-badge'
import { formatDateTime } from '@/lib/format'
import type { ProceedingDetail } from '@/lib/schemas'

interface StatusGridProps {
  proc: ProceedingDetail
}

export function StatusGrid({ proc }: StatusGridProps) {
  const { t } = useTranslation()
  return (
    <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
      <div className="rounded-lg border p-3">
        <div className="text-muted-foreground text-xs font-medium">
          {t('proceedings.colStav')}
        </div>
        <div className="mt-1">
          <StavBadge stav={proc.stav} />
        </div>
      </div>
      <div className="rounded-lg border p-3">
        <div className="text-muted-foreground text-xs font-medium">
          {t('proceedings.colUhrada')}
        </div>
        <div className="mt-1">
          <UhradaBadge stavUhrady={proc.stav_uhrady} label={proc.stav_uhrady_label} />
        </div>
      </div>
      <div className="rounded-lg border p-3">
        <div className="text-muted-foreground text-xs font-medium">
          {t('proceedings.datumPrijeti')}
        </div>
        <div className="mt-1 text-sm">
          {proc.datum_prijeti ? formatDateTime(proc.datum_prijeti) : '---'}
        </div>
      </div>
      <div className="rounded-lg border p-3">
        <div className="text-muted-foreground text-xs font-medium">
          {t('proceedings.lastCheck')}
        </div>
        <div className="mt-1 text-sm">
          {proc.last_check_at ? formatDateTime(proc.last_check_at) : '---'}
        </div>
      </div>
    </div>
  )
}
