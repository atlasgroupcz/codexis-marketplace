import { useTranslation } from 'react-i18next'
import { formatDate, formatTime, hasTime } from '@/lib/format'
import type { ProceedingDetail } from '@/lib/schemas'
import type { TimelineNode } from '@/lib/timeline'
import { deriveTimeline } from '@/lib/timeline'

interface ProceedingTimelineProps {
  proc: ProceedingDetail
}

const CHIP = 'rounded px-1 text-[10px] font-bold'

/**
 * One word on the node the proceeding currently rests at: what is happening, or
 * how it ended. Only the last node ever carries one - `deriveTimeline` marks a
 * single node either `current` or `isTerminal`, never both for the same reason.
 */
function StateChip({ node }: { node: TimelineNode }) {
  const { t } = useTranslation()
  if (node.isTerminal)
    return (
      <span
        className={`${CHIP} ${node.isNegative ? 'bg-red-100 text-red-700' : 'bg-stone-100 text-stone-600'}`}
      >
        {t(node.isNegative ? 'proceedings.timeline.rejected' : 'proceedings.timeline.ended')}
      </span>
    )
  if (node.state === 'current')
    return (
      <span className={`bg-brand-50 text-brand-700 ${CHIP}`}>{t('proceedings.timeline.now')}</span>
    )
  return null
}

function dotClass(node: TimelineNode): string {
  // Red is reserved for a genuinely bad outcome - rejected / stopped / withdrawn
  // (the only case `isNegative` is set, always on a terminal node).
  if (node.isNegative) return 'bg-destructive border-destructive'
  if (node.isPlomba) return 'bg-amber-400 border-amber-400' // caution, not alarm
  return 'bg-primary border-primary'
}

// The rail connects two events, so it belongs to the node above the gap - not to
// the list. On the <ol> it spanned the full block height, and because every node
// is taller than its dot, a stub always hung below the last one, reading as
// "something more is coming" on proceedings that had long since closed.
const RAIL =
  "before:bg-border before:absolute before:top-[18px] before:-bottom-[18px] before:-left-[25px] before:w-px before:content-[''] last:before:hidden"

/** Vertical log of what actually happened - a grey rail with one node per event. */
export function ProceedingTimeline({ proc }: ProceedingTimelineProps) {
  const nodes = deriveTimeline(proc)

  const nodeName = (n: TimelineNode) => n.rawName ?? ''

  return (
    <ol className="relative ml-1 space-y-4 pl-6">
      {nodes.map((n) => (
        <li
          key={n.key}
          data-testid={`timeline-node-${n.key}`}
          data-state={n.state}
          className={`relative ${RAIL}`}
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
              <StateChip node={n} />
            </div>
            {n.date !== null && (
              <div className="text-muted-foreground mt-0.5 text-xs tabular-nums">
                {formatDate(n.date)}
                {/* ČÚZK stamps each operation to the second, and several often
                    share a day - without the time they read as simultaneous. */}
                {hasTime(n.date) && (
                  <>
                    <span className="text-border"> · </span>
                    {formatTime(n.date)}
                  </>
                )}
              </div>
            )}
          </div>
        </li>
      ))}
    </ol>
  )
}
