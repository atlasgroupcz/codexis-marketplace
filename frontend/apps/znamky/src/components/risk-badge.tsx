import { useTranslation } from 'react-i18next'
import type { Tier } from '@/lib/schemas'

const TIER_CLASSES: Record<Tier, string> = {
  high: 'bg-red-100 text-red-800 ring-red-200 dark:bg-red-950/50 dark:text-red-200 dark:ring-red-900',
  medium:
    'bg-amber-100 text-amber-900 ring-amber-200 dark:bg-amber-950/50 dark:text-amber-200 dark:ring-amber-900',
  low: 'bg-slate-100 text-slate-700 ring-slate-200 dark:bg-slate-800 dark:text-slate-300 dark:ring-slate-700',
}

const DOT_CLASSES: Record<Tier, string> = {
  high: 'bg-red-500',
  medium: 'bg-amber-500',
  low: 'bg-slate-400',
}

export function RiskDot({ tier }: { tier: Tier | null | undefined }) {
  if (!tier) {
    return <span className="bg-muted size-2.5 shrink-0 rounded-full" aria-hidden />
  }
  return <span className={`size-2.5 shrink-0 rounded-full ${DOT_CLASSES[tier]}`} aria-hidden />
}

export function RiskBadge({ tier }: { tier: Tier }) {
  const { t } = useTranslation()
  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium ring-1 ${TIER_CLASSES[tier]}`}
      data-testid="risk-badge"
    >
      {t(`tier.${tier}`)}
    </span>
  )
}
