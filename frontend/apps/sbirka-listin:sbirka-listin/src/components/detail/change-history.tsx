import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from '@workspace/ui/components/sonner'
import type { Change } from '@/lib/schemas'
import { confirmChange } from '@/lib/api'
import { errorMessage, formatDateTime } from '@/lib/format'

interface ChangeHistoryProps {
  ico: string
  changes: Array<Change>
  onConfirmed: () => void
}

export function ChangeHistory({ ico, changes, onConfirmed }: ChangeHistoryProps) {
  const { t } = useTranslation()
  const [confirming, setConfirming] = useState(false)

  if (changes.length === 0) {
    return (
      <div className="flex flex-col items-center px-6 py-10 text-center" data-testid="no-changes">
        <p className="text-foreground text-sm font-medium">{t('subjekty.noChanges')}</p>
        <p className="text-muted-foreground mt-1 max-w-xs text-xs text-pretty">
          {t('subjekty.noChangesHint')}
        </p>
      </div>
    )
  }

  const unconfirmedCount = changes.filter((c) => !c.confirmed_on).length

  const handleConfirm = (index?: number) => {
    setConfirming(true)
    confirmChange(ico, index)
      .then(() => onConfirmed())
      .catch((e: unknown) => toast.error(errorMessage(e)))
      .finally(() => setConfirming(false))
  }

  // Newest first.
  const ordered = changes
    .map((change, index) => ({ change, index }))
    .sort((a, b) => (a.change.detected_on < b.change.detected_on ? 1 : -1))

  return (
    <div className="space-y-2.5">
      {unconfirmedCount > 0 && (
        <div className="flex justify-end">
          <button
            type="button"
            onClick={() => handleConfirm()}
            disabled={confirming}
            className="text-muted-foreground hover:text-foreground rounded-md border px-2.5 py-1 text-xs"
            data-testid="mark-all-read"
          >
            {t('subjekty.markAllRead', { count: unconfirmedCount })}
          </button>
        </div>
      )}

      {ordered.map(({ change, index }) => {
        const isUnread = !change.confirmed_on
        return (
          <div
            key={index}
            className={`rounded-[10px] border p-3.5 ${
              isUnread ? 'border-brand-200 bg-secondary' : 'border-border bg-card'
            }`}
          >
            <div className="mb-2 flex items-center justify-between gap-2.5">
              <span className="text-muted-foreground flex items-center gap-2 text-[12.5px]">
                {isUnread && <span aria-hidden className="bg-primary size-[7px] shrink-0 rounded-full" />}
                {formatDateTime(change.detected_on)}
              </span>
              {isUnread && (
                <button
                  type="button"
                  onClick={() => handleConfirm(index)}
                  disabled={confirming}
                  className="text-brand-700 border-brand-200 hover:bg-background shrink-0 rounded-md border px-2 py-0.5 text-[11.5px] font-semibold"
                >
                  {t('subjekty.markRead')}
                </button>
              )}
            </div>
            {change.nove_listiny.map((l, j) => (
              <p key={`n${j}`} className="py-0.5 text-[13px] leading-relaxed">
                <span className="text-primary font-extrabold">+</span> {l.typ}{' '}
                <span className="text-muted-foreground font-mono text-[11.5px]">{l.cislo}</span>
              </p>
            ))}
            {change.zmizele_listiny.map((cislo, j) => (
              <p key={`z${j}`} className="text-muted-foreground py-0.5 text-[11.5px]">
                &minus; {t('subjekty.vanishedListina')}: {cislo}
              </p>
            ))}
          </div>
        )
      })}
    </div>
  )
}
