import { Badge } from '@workspace/ui/components/badge'
import { useTranslation } from 'react-i18next'
import { TrackingTypeBadge } from './tracking-type-badge'
import type { TrackedDocumentSummary } from '@/lib/schemas'
import { formatDate } from '@/lib/format'

interface DocumentListItemProps {
  document: TrackedDocumentSummary
  onClick: (uuid: string) => void
}

export function DocumentListItem({ document, onClick }: DocumentListItemProps) {
  const { t } = useTranslation()

  return (
    <button
      type="button"
      className="bg-card text-card-foreground flex w-full items-center gap-4 rounded-xl border p-4 text-left shadow-sm transition-colors hover:bg-accent/50"
      onClick={() => onClick(document.uuid)}
    >
      <div className="flex-1 space-y-1">
        <div className="font-semibold">{document.name}</div>
        <div className="text-muted-foreground flex items-center gap-2 text-sm">
          <span>{document.codexisId}</span>
          <span>&middot;</span>
          <span>{formatDate(document.added_on)}</span>
        </div>
      </div>
      <TrackingTypeBadge type={document.tracking_type} />
      {document.unconfirmed_changes > 0 && (
        <Badge variant="destructive">
          {t('followedDocs.unconfirmed', { count: document.unconfirmed_changes })}
        </Badge>
      )}
      <Badge variant="secondary">
        {t('followedDocs.total', { count: document.total_changes })}
      </Badge>
    </button>
  )
}
