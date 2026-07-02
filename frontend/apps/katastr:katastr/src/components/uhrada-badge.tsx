interface UhradaBadgeProps {
  stavUhrady: string | null | undefined
  label: string | null | undefined
}

// Coloured text (like the mockup's `.pay`), not a pill.
const colorMap: Record<string, string> = {
  U: 'text-brand-700',
  N: 'text-plomba',
  O: 'text-amber-700',
}

export function UhradaBadge({ stavUhrady, label }: UhradaBadgeProps) {
  if (!stavUhrady || !label) {
    return <span className="text-muted-foreground">---</span>
  }
  const color = colorMap[stavUhrady] ?? 'text-foreground'
  return <span className={`font-semibold ${color}`}>{label}</span>
}
