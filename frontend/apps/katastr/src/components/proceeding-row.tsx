import { StavBadge } from './stav-badge'
import type { ProceedingSummary } from '@/lib/schemas'

interface ProceedingRowProps {
  proceeding: ProceedingSummary
  selected: boolean
  onClick: (cisloRizeni: string) => void
}

export function ProceedingRow({ proceeding, selected, onClick }: ProceedingRowProps) {
  const hasUnread = proceeding.unconfirmed_count > 0
  return (
    <button
      type="button"
      onClick={() => onClick(proceeding.cislo_rizeni)}
      data-testid={`proceeding-row-${proceeding.cislo_rizeni}`}
      aria-current={selected}
      className={`flex w-full items-start gap-2.5 border-b px-3 py-2.5 text-left transition-colors last:border-b-0 ${
        selected ? 'bg-muted shadow-[inset_3px_0_0_var(--primary)]' : 'hover:bg-muted/60'
      }`}
    >
      <span
        data-testid={hasUnread ? 'unread-dot' : undefined}
        aria-hidden
        className={`mt-1.5 size-2 shrink-0 rounded-full ${hasUnread ? 'bg-primary' : 'bg-transparent'}`}
      />
      <span className="min-w-0 flex-1">
        <span className="block truncate font-mono text-[13px] font-semibold">
          {proceeding.cislo_rizeni}
        </span>
        <span className="mt-1.5 flex min-w-0 items-center gap-2">
          <span className="shrink-0">
            <StavBadge stav={proceeding.stav} />
          </span>
          {proceeding.label && (
            <span
              data-testid="proceeding-label"
              className="text-muted-foreground min-w-0 flex-1 truncate text-xs"
            >
              {proceeding.label}
            </span>
          )}
        </span>
      </span>
    </button>
  )
}
