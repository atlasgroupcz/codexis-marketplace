import { FileText } from 'lucide-react'
import type { SubjektSummary } from '@/lib/schemas'

interface SubjektRowProps {
  subjekt: SubjektSummary
  selected: boolean
  onClick: (ico: string) => void
}

export function SubjektRow({ subjekt, selected, onClick }: SubjektRowProps) {
  const hasUnread = subjekt.nepotvrzenych_zmen > 0
  const name = subjekt.nazev || subjekt.ico
  return (
    <button
      type="button"
      onClick={() => onClick(subjekt.ico)}
      data-testid={`subjekt-row-${subjekt.ico}`}
      aria-current={selected}
      className={`flex w-full items-center gap-2.5 border-b px-3 py-2.5 text-left transition-colors last:border-b-0 ${
        selected ? 'bg-background shadow-[inset_3px_0_0_var(--primary)]' : 'hover:bg-background/60'
      }`}
    >
      {/* Leading dot = unread (distinct from the inset bar, which marks selection). */}
      <span
        data-testid={hasUnread ? 'unread-dot' : undefined}
        aria-hidden
        className={`size-2 shrink-0 rounded-full ${hasUnread ? 'bg-primary' : 'bg-transparent'}`}
      />
      <span className="min-w-0 flex-1">
        <span className="block truncate text-[13px] font-semibold">{name}</span>
        <span className="text-muted-foreground block font-mono text-xs">IČO {subjekt.ico}</span>
      </span>
      <span className="text-muted-foreground flex shrink-0 items-center gap-1 text-[11px] tabular-nums">
        <FileText className="size-3.5" strokeWidth={1.75} />
        {subjekt.listin}
      </span>
    </button>
  )
}
