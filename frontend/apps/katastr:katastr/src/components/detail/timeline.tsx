import { useTranslation } from 'react-i18next'
import type { ProceedingDetail } from '@/lib/schemas'
import type { TimelineNode } from '@/lib/timeline'
import { deriveTimeline } from '@/lib/timeline'
import { formatDate } from '@/lib/format'

interface ProceedingTimelineProps {
  proc: ProceedingDetail
}

function dotClass(node: TimelineNode): string {
  // Red is reserved for a genuinely bad outcome — rejected / stopped / withdrawn.
  if (node.isTerminal && node.isNegative) return 'bg-destructive border-destructive'
  if (node.isTerminal) return 'bg-primary border-primary'
  if (node.isPlomba) return 'bg-amber-400 border-amber-400' // caution, not alarm
  return 'bg-primary border-primary'
}

/** Vertical log of what actually happened — a grey rail with one node per event. */
export function ProceedingTimeline({ proc }: ProceedingTimelineProps) {
  const { t } = useTranslation()
  const nodes = deriveTimeline(proc)

  const nodeName = (n: TimelineNode) => n.rawName ?? ''

  return (
    <ol className="relative ml-1 space-y-4 border-l pl-6">
      {nodes.map((n) => (
        <li
          key={n.key}
          data-testid={`timeline-node-${n.key}`}
          data-state={n.state}
          className="relative"
        >
          <span
            aria-hidden
            className={`absolute top-0.5 -left-[31px] size-3.5 rounded-full border-2 ${dotClass(n)} ${
              n.state === 'current' && !n.isTerminal ? 'ring-brand-50 ring-4' : ''
            }`}
          />
          <div className="min-w-0">
            <div className="flex items-center gap-1.5 text-[13px] leading-tight font-semibold">
              <span className="truncate">{nodeName(n)}</span>
              {n.isPlomba && <span className="text-amber-500">&#11041;</span>}
              {n.state === 'current' && !n.isTerminal && (
                <span className="bg-brand-50 text-brand-700 rounded px-1 text-[10px] font-bold">
                  {t('proceedings.timeline.now')}
                </span>
              )}
            </div>
            {n.date !== null && (
              <div className="text-muted-foreground mt-0.5 text-xs tabular-nums">
                {formatDate(n.date)}
              </div>
            )}
          </div>
        </li>
      ))}
    </ol>
  )
}
