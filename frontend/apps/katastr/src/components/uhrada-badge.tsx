import { useTranslation } from 'react-i18next'
import { uhradaLabel } from '@/lib/uhrada'

interface UhradaBadgeProps {
  stavUhrady: string | null | undefined
}

// Coloured text (like the mockup's `.pay`), not a pill.
const colorMap: Record<string, string> = {
  U: 'text-brand-700',
  N: 'text-plomba',
  O: 'text-amber-700',
}

/** The payment state of a proceeding, coloured, as the detail header shows it. */
export function UhradaBadge({ stavUhrady }: UhradaBadgeProps) {
  const { t } = useTranslation()
  if (!stavUhrady) return <span className="text-muted-foreground">---</span>
  const color = colorMap[stavUhrady] ?? 'text-foreground'
  return <span className={`font-semibold ${color}`}>{uhradaLabel(t, stavUhrady)}</span>
}
