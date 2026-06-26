import { Badge } from '@workspace/ui/components/badge'

interface StavBadgeProps {
  stav: string
}

// Keyword → tailwind classes. Order matters: most specific first.
function getStavColor(stav: string): string | null {
  const s = stav.toLowerCase()
  if (s.includes('ukončeno')) return 'bg-stone-100 text-stone-600'
  if (s.includes('vyrozumění') || s.includes('provedení vkladu'))
    return 'bg-brand-100 text-brand-700'
  if (s.includes('plomb')) return 'bg-plomba-soft text-plomba'
  if (
    s.includes('povolení vkladu') ||
    s.includes('probíhá') ||
    s.includes('podáno') ||
    s.includes('přijet')
  )
    return 'bg-amber-100 text-amber-800'
  return null
}

// Verbose ČÚZK statuses → short chip label. Order matters: vyrozumění before
// provedení (a vyrozumění string also contains "proveden"). Falls back to the
// raw stav for anything we don't recognise, so nothing is hidden.
function shortStav(stav: string): string {
  const s = stav.toLowerCase()
  if (s.includes('ukončeno')) return 'Ukončeno'
  if (s.includes('vyrozum')) return 'Vyrozumění'
  if (s.includes('provedení vkladu') || s.includes('proveden')) return 'Provedení vkladu'
  if (s.includes('povolení vkladu') || s.includes('povolen')) return 'Povolení vkladu'
  if (s.includes('plomb')) return 'Plomba'
  if (s.includes('probíhá')) return 'Probíhá'
  if (s.includes('přijet') || s.includes('podáno')) return 'Přijetí návrhu'
  return stav
}

const PILL_BASE =
  'inline-flex items-center whitespace-nowrap rounded-md px-1.5 py-0.5 text-[10.5px] font-semibold'

export function StavBadge({ stav }: StavBadgeProps) {
  const colorClass = getStavColor(stav)
  const label = shortStav(stav)
  if (colorClass) {
    return (
      <span title={stav} className={`${PILL_BASE} ${colorClass}`}>
        {label}
      </span>
    )
  }
  return (
    <Badge variant="outline" title={stav} className="text-[10.5px] font-semibold">
      {label}
    </Badge>
  )
}
