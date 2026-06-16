import { useTranslation } from 'react-i18next'
import { ShieldAlert, ShieldCheck } from 'lucide-react'

const DRUH_CLASSES: Record<string, string> = {
  konkurs:
    'bg-red-100 text-red-800 dark:bg-red-950 dark:text-red-200 ring-red-200 dark:ring-red-900',
  reorganizace:
    'bg-amber-100 text-amber-900 dark:bg-amber-950 dark:text-amber-200 ring-amber-200 dark:ring-amber-900',
  moratorium:
    'bg-amber-100 text-amber-900 dark:bg-amber-950 dark:text-amber-200 ring-amber-200 dark:ring-amber-900',
  oddluzeni:
    'bg-blue-100 text-blue-800 dark:bg-blue-950 dark:text-blue-200 ring-blue-200 dark:ring-blue-900',
  neurceno:
    'bg-muted text-muted-foreground ring-border',
}

/** Coloured pill for an insolvency-proceeding type (druh). */
export function DruhBadge({ druh }: { druh: string }) {
  const { t } = useTranslation()
  const cls = DRUH_CLASSES[druh] ?? DRUH_CLASSES.neurceno
  return (
    <span
      className={`inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium ring-1 ring-inset ${cls}`}
    >
      {t(`druh.${druh}`, { defaultValue: t('druh.neurceno') })}
    </span>
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
      <span
        className={`inline-flex items-center gap-1.5 font-medium text-red-700 dark:text-red-300 ${text}`}
      >
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
    <span
      className={`inline-flex items-center gap-1.5 font-medium text-emerald-700 dark:text-emerald-400 ${text}`}
    >
      <ShieldCheck className={iconSize} />
      {t('status.clean')}
    </span>
  )
}

/** Small status dot for the master list. */
export function StatusDot({ active, hasHistory }: { active: boolean; hasHistory: boolean }) {
  const color = active
    ? 'bg-red-500'
    : hasHistory
      ? 'bg-muted-foreground/40'
      : 'bg-emerald-500'
  return <span className={`size-2.5 shrink-0 rounded-full ${color}`} aria-hidden />
}
