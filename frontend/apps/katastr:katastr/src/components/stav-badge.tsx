import { Badge } from '@workspace/ui/components/badge'

interface StavBadgeProps {
  stav: string
}

function getStavVariant(stav: string): 'default' | 'secondary' | 'destructive' | 'outline' {
  const lower = stav.toLowerCase()
  if (lower.includes('ukončeno')) return 'secondary'
  if (lower.includes('vyrozumění o provedeném vkladu') || lower.includes('odesláno vyrozumění')) return 'secondary'
  if (lower.includes('probíhá') || lower.includes('plomb') || lower.includes('zaplomb')) return 'destructive'
  if (lower.includes('povolení vkladu') || lower.includes('provedení vkladu')) return 'default'
  return 'outline'
}

function getStavColor(stav: string): string {
  const lower = stav.toLowerCase()
  if (lower.includes('ukončeno')) return 'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300'
  if (lower.includes('vyrozumění o provedeném vkladu') || lower.includes('odesláno vyrozumění'))
    return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
  if (lower.includes('probíhá'))
    return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
  if (lower.includes('plomb') || lower.includes('informace o vyznačení plomby'))
    return 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200'
  return ''
}

export function StavBadge({ stav }: StavBadgeProps) {
  const colorClass = getStavColor(stav)

  if (colorClass) {
    return (
      <span className={`inline-flex items-center rounded-md px-2 py-1 text-xs font-medium ${colorClass}`}>
        {stav}
      </span>
    )
  }

  return (
    <Badge variant={getStavVariant(stav)} className="text-xs font-medium">
      {stav}
    </Badge>
  )
}
