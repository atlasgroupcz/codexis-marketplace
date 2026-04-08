import { useState } from 'react'
import { BellOff } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@workspace/ui/components/button'
import { confirmChange } from '@/lib/api'
import { formatDateTime } from '@/lib/format'
import type { ProceedingChange } from '@/lib/schemas'

interface ChangeHistoryProps {
  uuid: string
  changes: ProceedingChange[]
  onConfirmed: () => void
}

export function ChangeHistory({ uuid, changes, onConfirmed }: ChangeHistoryProps) {
  const { t } = useTranslation()
  const [confirming, setConfirming] = useState(false)

  if (changes.length === 0) return null

  const unconfirmedCount = changes.filter((c) => !c.confirmed_on).length

  const handleConfirmAll = () => {
    setConfirming(true)
    confirmChange(uuid)
      .then(() => onConfirmed())
      .finally(() => setConfirming(false))
  }

  const handleConfirmOne = (index: number) => {
    setConfirming(true)
    confirmChange(uuid, index)
      .then(() => onConfirmed())
      .finally(() => setConfirming(false))
  }

  return (
    <div>
      <div className="mb-3 flex items-center justify-between">
        <h2 className="text-lg font-semibold">{t('proceedings.changeHistory')}</h2>
        {unconfirmedCount > 0 && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleConfirmAll}
            disabled={confirming}
          >
            <BellOff className="size-4" />
            {t('proceedings.markAllRead', { count: unconfirmedCount })}
          </Button>
        )}
      </div>
      <div className="space-y-2">
        {changes.map((change, i) => {
          const isUnread = !change.confirmed_on
          return (
            <div
              key={i}
              className={`rounded-lg border p-3 ${
                isUnread
                  ? 'border-blue-300 bg-blue-50/50 dark:border-blue-800 dark:bg-blue-950/30'
                  : ''
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="text-muted-foreground text-xs">
                  {formatDateTime(change.detected_on)}
                  {!isUnread && change.confirmed_on && (
                    <span className="ml-2">
                      ({t('proceedings.readOn', { date: formatDateTime(change.confirmed_on) })})
                    </span>
                  )}
                </div>
                {isUnread && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleConfirmOne(i)}
                    disabled={confirming}
                  >
                    {t('proceedings.markRead')}
                  </Button>
                )}
              </div>
              {change.old_stav !== change.new_stav && (
                <div className="mt-1 text-sm">
                  <span className="text-muted-foreground">{change.old_stav}</span>
                  <span className="mx-2">&rarr;</span>
                  <span className="font-medium">{change.new_stav}</span>
                </div>
              )}
              {change.new_operations.map((op, j) => (
                <div key={j} className="mt-1 text-xs">
                  + {op.nazev} ({formatDateTime(op.datumProvedeni)})
                </div>
              ))}
              {change.stav_uhrady_changed && (
                <div className="mt-1 text-xs">
                  {t('proceedings.uhradaChanged')}: {change.old_stav_uhrady ?? '---'} &rarr;{' '}
                  {change.new_stav_uhrady ?? '---'}
                </div>
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
}
