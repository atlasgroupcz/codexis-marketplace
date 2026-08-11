import { Badge } from '@workspace/ui/components/badge'
import { ShieldAlert, ShieldCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'

const DRUH_CLASSES: Record<string, string> = {
  konkurs: 'border-red-200 bg-red-50 text-red-800',
  reorganizace: 'border-amber-200 bg-amber-50 text-amber-900',
  moratorium: 'border-amber-200 bg-amber-50 text-amber-900',
  oddluzeni: 'border-blue-200 bg-blue-50 text-blue-800',
  neurceno: 'border-border bg-muted text-muted-foreground',
}

/** Coloured pill for an insolvency-proceeding type (druh). */
export function DruhBadge({ druh }: { druh: string }) {
  const { t } = useTranslation()
  const cls = DRUH_CLASSES[druh] ?? DRUH_CLASSES.neurceno
  return (
    <Badge variant="outline" className={`rounded-md font-medium ${cls}`}>
      {t(`druh.${druh}`, { defaultValue: t('druh.neurceno') })}
    </Badge>
  )
}

/** Subject-level insolvency state used in list items and the detail header. */
export function InsolvencyStatus({
  active,
  hasHistory,
  size = 'sm',
}: {
  active: boolean
  hasHistory: boolean
  size?: 'sm' | 'md'
}) {
  const { t } = useTranslation()
  const iconSize = size === 'md' ? 'size-4' : 'size-3.5'
  const text = size === 'md' ? 'text-sm' : 'text-xs'

  if (active) {
    return (
      <span className={`inline-flex items-center gap-1.5 font-medium text-red-700 ${text}`}>
        <ShieldAlert className={iconSize} />
        {t('status.active')}
      </span>
    )
  }
  if (hasHistory) {
    return (
      <span className={`text-muted-foreground inline-flex items-center gap-1.5 ${text}`}>
        <ShieldCheck className={iconSize} />
        {t('status.closed')}
      </span>
    )
  }
  return (
    <span className={`text-brand-700 inline-flex items-center gap-1.5 font-medium ${text}`}>
      <ShieldCheck className={iconSize} />
      {t('status.clean')}
    </span>
  )
}

/** Small status dot for the master list. */
export function StatusDot({ active, hasHistory }: { active: boolean; hasHistory: boolean }) {
  const color = active ? 'bg-red-500' : hasHistory ? 'bg-muted-foreground/40' : 'bg-brand-500'
  return <span className={`size-2.5 shrink-0 rounded-full ${color}`} aria-hidden />
}
