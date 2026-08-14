import { Badge } from '@workspace/ui/components/badge'
import { useTranslation } from 'react-i18next'
import type { StavKind } from '@/lib/stav'
import { stavKind } from '@/lib/stav'
import { CLOSED_CLASSES, NEGATIVE_CLASSES } from './stav-colors'

interface StavBadgeProps {
  stav: string
}

// Several states share a colour - approved, running and received are all "under
// way" - so the map is wider than the palette. `stavKind` decides which state it
// is; this decides only how it looks.
const KIND_CLASSES: Record<StavKind, string> = {
  negative: NEGATIVE_CLASSES,
  mistaken: CLOSED_CLASSES,
  closed: CLOSED_CLASSES,
  notified: 'bg-brand-100 text-brand-700',
  registered: 'bg-brand-100 text-brand-700',
  allowed: 'bg-amber-100 text-amber-800',
  plomba: 'bg-plomba-soft text-plomba',
  running: 'bg-amber-100 text-amber-800',
  received: 'bg-amber-100 text-amber-800',
}

const PILL_BASE =
  'inline-flex items-center whitespace-nowrap rounded-md px-1.5 py-0.5 text-[10.5px] font-semibold'

export function StavBadge({ stav }: StavBadgeProps) {
  const { t } = useTranslation()
  const kind = stavKind(stav)

  // ČÚZK writes `stav` as a sentence and only a few characters fit, so a state
  // we recognise wears our own short name for it - in the reader's language,
  // which the wire text never is. The full sentence stays in the tooltip.
  //
  // A bad ending is the exception: "Zamítnuto", "Zastaveno" and "Zpětvzetí" are
  // three different outcomes, each already short, and one shared word for them
  // would say less than ČÚZK did.
  const label = kind && kind !== 'negative' ? t(`proceedings.stav.${kind}`) : stav

  if (!kind) {
    return (
      <Badge variant="outline" title={stav} className="text-[10.5px] font-semibold">
        {label}
      </Badge>
    )
  }
  return (
    <span title={stav} className={`${PILL_BASE} ${KIND_CLASSES[kind]}`}>
      {label}
    </span>
  )
}
