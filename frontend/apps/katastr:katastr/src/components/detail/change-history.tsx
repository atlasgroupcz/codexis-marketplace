import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Card, CardContent } from '@workspace/ui/components/card'
import { toast } from '@workspace/ui/components/sonner'
import type { ProceedingChange } from '@/lib/schemas'
import { confirmChange } from '@/lib/api'
import { errorMessage, formatDateTime } from '@/lib/format'

interface ChangeHistoryProps {
  uuid: string
  changes: Array<ProceedingChange>
  onConfirmed: () => void
}

export function ChangeHistory({ uuid, changes, onConfirmed }: ChangeHistoryProps) {
  const { t } = useTranslation()
  const [confirming, setConfirming] = useState(false)

  if (changes.length === 0) {
    return (
      <div className="flex flex-col items-center px-6 py-10 text-center" data-testid="no-changes">
        <p className="text-foreground text-sm font-medium">{t('proceedings.noChanges')}</p>
        <p className="text-muted-foreground mt-1 max-w-xs text-xs text-pretty">
          {t('proceedings.noChangesHint')}
        </p>
      </div>
    )
  }

  const unconfirmedCount = changes.filter((c) => !c.confirmed_on).length

  const handleConfirm = (index?: number) => {
    setConfirming(true)
    confirmChange(uuid, index)
      .then(() => onConfirmed())
      .catch((e: unknown) => toast.error(errorMessage(e)))
      .finally(() => setConfirming(false))
  }

  // Newest first.
  const ordered = changes
    .map((change, index) => ({ change, index }))
    .sort((a, b) => (a.change.detected_on < b.change.detected_on ? 1 : -1))

  return (
    <div className="space-y-3">
      {unconfirmedCount > 0 && (
        <div className="flex justify-end">
          <button
            type="button"
            onClick={() => handleConfirm()}
            disabled={confirming}
            className="text-muted-foreground hover:text-foreground rounded-md border px-2.5 py-1 text-xs"
            data-testid="mark-all-read"
          >
            {t('proceedings.markAllRead', { count: unconfirmedCount })}
          </button>
        </div>
      )}

      <ol className="relative space-y-3 border-l pl-5">
        {ordered.map(({ change, index }) => {
          const isUnread = !change.confirmed_on
          return (
            <li key={index} className="relative">
              <span
                aria-hidden
                className={`bg-background absolute top-1 -left-[26px] size-3 rounded-full border-2 ${
                  isUnread ? 'border-primary bg-primary' : 'border-border'
                }`}
              />
              <Card className={`gap-0 py-0 ${isUnread ? 'border-primary/40' : ''}`}>
                <CardContent className="p-3">
                  <div className="flex items-start justify-between gap-2">
                    <time className="text-muted-foreground text-xs">
                      {formatDateTime(change.detected_on)}
                    </time>
                    {isUnread && (
                      <button
                        type="button"
                        onClick={() => handleConfirm(index)}
                        disabled={confirming}
                        className="text-brand-700 shrink-0 text-[11px] hover:underline"
                      >
                        {t('proceedings.markRead')}
                      </button>
                    )}
                  </div>
                  {change.old_stav !== change.new_stav && (
                    <p className="mt-1 text-[13px]">
                      <span className="text-muted-foreground">{change.old_stav}</span>
                      <span className="mx-1.5 text-[#bdbdb8]">&rarr;</span>
                      <span>{change.new_stav}</span>
                    </p>
                  )}
                  {change.new_operations.map((op, j) => (
                    <p key={j} className="text-muted-foreground mt-0.5 text-[11px]">
                      + {op.nazev} ({formatDateTime(op.datumProvedeni)})
                    </p>
                  ))}
                  {change.stav_uhrady_changed && (
                    <p className="text-muted-foreground mt-0.5 text-[11px]">
                      {t('proceedings.uhradaChanged')}: {change.old_stav_uhrady ?? '---'} &rarr;{' '}
                      {change.new_stav_uhrady ?? '---'}
                    </p>
                  )}
                </CardContent>
              </Card>
            </li>
          )
        })}
      </ol>
    </div>
  )
}
