import { useState } from 'react'
import Markdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Check, ExternalLink } from 'lucide-react'
import { Badge } from '@workspace/ui/components/badge'
import { Button } from '@workspace/ui/components/button'
import { useTranslation } from 'react-i18next'
import { ChangeTypeBadge } from './change-type-badge'
import type { Change } from '@/lib/schemas'
import { formatDate, formatDateTime } from '@/lib/format'
import { postAction } from '@/lib/api'

interface ChangeItemProps {
  change: Change
  changeIndex: number
  uuid: string
  onMutate: () => void
}

export function ChangeItem({ change, changeIndex, uuid, onMutate }: ChangeItemProps) {
  const { t } = useTranslation()
  const [amendmentsExpanded, setAmendmentsExpanded] = useState(false)
  const [confirming, setConfirming] = useState(false)
  const maxVisibleAmendments = 3

  const handleConfirm = (): void => {
    setConfirming(true)
    postAction('confirm', uuid, changeIndex)
      .then(() => onMutate())
      .catch(() => setConfirming(false))
  }

  return (
    <div className="space-y-3 rounded-xl border p-4">
      <div className="flex flex-wrap items-center gap-2">
        <ChangeTypeBadge type={change.change_type} />
        {change.confirmed_on ? (
          <Badge variant="outline">
            {t('followedDocs.confirmed', {
              date: formatDateTime(change.confirmed_on),
            })}
          </Badge>
        ) : (
          <>
            <Badge variant="destructive">{t('followedDocs.notConfirmed')}</Badge>
            <Button
              variant="outline"
              size="sm"
              className="h-6 gap-1 px-2 text-xs"
              disabled={confirming}
              onClick={handleConfirm}
            >
              <Check className="size-3" />
              {confirming ? t('followedDocs.confirming') : t('followedDocs.confirm')}
            </Button>
          </>
        )}
      </div>

      <div className="text-muted-foreground flex flex-wrap gap-x-4 gap-y-1 text-sm">
        <span>
          {t('followedDocs.detected', { date: formatDateTime(change.detected_on) })}
        </span>
        <span>
          {t('followedDocs.effective', { date: formatDate(change.effective_on) })}
        </span>
      </div>

      {change.description_md && (
        <div className="prose prose-sm dark:prose-invert max-w-none font-semibold">
          <Markdown remarkPlugins={[remarkGfm]}>
            {change.description_md}
          </Markdown>
        </div>
      )}

      {change.amendments.length > 0 && (
        <div className="text-muted-foreground text-xs">
          <span>{t('followedDocs.amendments')}{' '}</span>
          {change.amendments.slice(0, amendmentsExpanded ? undefined : maxVisibleAmendments).map((a, i) => (
            <span key={a.id}>
              {i > 0 && ', '}
              {a.name}
            </span>
          ))}
          {change.amendments.length > maxVisibleAmendments && (
            <button
              type="button"
              className="text-foreground/60 hover:text-foreground ml-1 underline transition-colors"
              onClick={() => setAmendmentsExpanded(!amendmentsExpanded)}
            >
              {amendmentsExpanded
                ? t('followedDocs.showLess')
                : t('followedDocs.showMore', { count: change.amendments.length - maxVisibleAmendments })}
            </button>
          )}
        </div>
      )}

      {change.compare_url && (
        <a
          href={change.compare_url}
          target="_blank"
          rel="noopener noreferrer"
          className="text-primary hover:text-primary/80 inline-flex items-center gap-1.5 text-sm font-medium transition-colors"
        >
          <ExternalLink className="size-4" />
          {t('followedDocs.viewInCodexis')}
        </a>
      )}
    </div>
  )
}
