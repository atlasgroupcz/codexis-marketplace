import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Activity, BellOff, FileText, ShieldAlert } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { toast } from '@workspace/ui/components/sonner'
import type { SubjectChange } from '@/lib/schemas'
import { formatDateTime } from '@/lib/format'
import { confirmChange } from '@/lib/api'

interface ChangeTimelineProps {
  uuid: string
  changes: Array<SubjectChange>
  onConfirmed: () => void
}

function iconFor(typ: string) {
  if (typ === 'nove_rizeni')
    return <ShieldAlert className="size-4 text-red-600 dark:text-red-400" />
  if (typ === 'nova_udalost')
    return <FileText className="size-4 text-blue-600 dark:text-blue-400" />
  return <Activity className="size-4 text-amber-600 dark:text-amber-400" />
}

export function ChangeTimeline({ uuid, changes, onConfirmed }: ChangeTimelineProps) {
  const { t } = useTranslation()
  const [busy, setBusy] = useState(false)

  if (changes.length === 0) return null

  // Keep original indices (confirm targets them) but show newest first.
  const ordered = changes
    .map((change, index) => ({ change, index }))
    .sort((a, b) => (a.change.detected_on < b.change.detected_on ? 1 : -1))
  const unconfirmed = changes.filter((c) => !c.confirmed_on).length

  const run = (fn: () => Promise<unknown>) => {
    setBusy(true)
    fn()
      .then(() => onConfirmed())
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setBusy(false))
  }

  return (
    <div>
      <div className="mb-3 flex items-center justify-between">
        <h2 className="text-sm font-semibold">{t('detail.changeHistory')}</h2>
        {unconfirmed > 0 && (
          <Button
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={() => run(() => confirmChange(uuid))}
            data-testid="mark-all-read"
          >
            <BellOff className="size-4" />
            {t('detail.markAllRead', { count: unconfirmed })}
          </Button>
        )}
      </div>
      <ol className="relative space-y-3 border-l pl-5">
        {ordered.map(({ change, index }) => {
          const unread = !change.confirmed_on
          return (
            <li key={index} className="relative">
              <span className="bg-background absolute top-0.5 -left-[26px] flex size-6 items-center justify-center rounded-full border">
                {iconFor(change.typ)}
              </span>
              <div
                className={`rounded-lg border p-3 ${
                  unread
                    ? 'border-blue-300 bg-blue-50/60 dark:border-blue-900 dark:bg-blue-950/30'
                    : ''
                }`}
              >
                <div className="flex items-center justify-between gap-2">
                  <time className="text-muted-foreground text-xs">
                    {formatDateTime(change.detected_on)}
                  </time>
                  {unread && (
                    <Button
                      variant="ghost"
                      size="sm"
                      disabled={busy}
                      onClick={() => run(() => confirmChange(uuid, index))}
                    >
                      {t('detail.markRead')}
                    </Button>
                  )}
                </div>
                <p className="mt-1 text-sm">{change.popis}</p>
              </div>
            </li>
          )
        })}
      </ol>
    </div>
  )
}
