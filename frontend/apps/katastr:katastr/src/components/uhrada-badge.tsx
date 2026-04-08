interface UhradaBadgeProps {
  stavUhrady: string | null | undefined
  label: string | null | undefined
}

export function UhradaBadge({ stavUhrady, label }: UhradaBadgeProps) {
  if (!stavUhrady || !label) {
    return <span className="text-muted-foreground text-sm">---</span>
  }

  const colorMap: Record<string, string> = {
    U: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
    N: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
    O: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  }

  const color = colorMap[stavUhrady] ?? 'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300'

  return (
    <span className={`inline-flex items-center rounded-md px-2 py-1 text-xs font-medium ${color}`}>
      {label}
    </span>
  )
}
