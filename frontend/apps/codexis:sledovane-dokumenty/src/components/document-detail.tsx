import { ArrowLeft } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Badge } from '@workspace/ui/components/badge'
import { useTranslation } from 'react-i18next'
import { TrackingTypeBadge } from './tracking-type-badge'
import { ChangeItem } from './change-item'
import { DetailSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { useDocumentDetail } from '@/hooks/use-document-detail'
import { formatDate } from '@/lib/format'

interface DocumentDetailProps {
  uuid: string
  onBack: () => void
}

export function DocumentDetail({ uuid, onBack }: DocumentDetailProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useDocumentDetail(uuid)

  if (loading && !data) {
    return <DetailSkeleton />
  }

  if (error) {
    return <ErrorMessage message={error.message} onRetry={refetch} />
  }

  if (!data) {
    return null
  }

  const { document } = data

  return (
    <div className="space-y-6 p-6">
      <Button variant="ghost" size="sm" onClick={onBack}>
        <ArrowLeft className="size-4" />
        {t('followedDocs.back')}
      </Button>

      <div className="space-y-2">
        <h1 className="text-xl font-semibold">{document.name}</h1>
        <div className="text-muted-foreground flex flex-wrap items-center gap-2 text-sm">
          <span>{document.codexisId}</span>
          <span>&middot;</span>
          <span>
            {t('followedDocs.trackedSince', { date: formatDate(document.added_on) })}
          </span>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <TrackingTypeBadge type={document.tracking_type} />
          <Badge variant="secondary">
            {t('followedDocs.total', { count: document.total_changes })}
          </Badge>
          {document.unconfirmed_changes > 0 && (
            <Badge variant="destructive">
              {t('followedDocs.unconfirmed', { count: document.unconfirmed_changes })}
            </Badge>
          )}
        </div>
      </div>

      {document.parts.length > 0 && (
        <div className="space-y-1">
          <h2 className="text-sm font-medium">{t('followedDocs.trackedParts')}</h2>
          <div className="flex flex-wrap gap-1">
            {document.parts.map((part) => (
              <Badge key={part.partId} variant="outline">{part.label}</Badge>
            ))}
          </div>
        </div>
      )}

      <div className="space-y-3">
        <h2 className="text-sm font-medium">
          {t('followedDocs.changes', { count: document.changes.length })}
        </h2>
        {document.changes.map((change, index) => (
          <ChangeItem key={index} change={change} />
        ))}
      </div>
    </div>
  )
}
