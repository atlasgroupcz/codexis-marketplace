import { useTranslation } from 'react-i18next'
import { DocumentListItem } from './document-list-item'
import { ListSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { useOverview } from '@/hooks/use-overview'

interface DocumentListProps {
  onSelectDocument: (uuid: string) => void
}

export function DocumentList({ onSelectDocument }: DocumentListProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()

  if (loading && !data) {
    return <ListSkeleton />
  }

  if (error) {
    return <ErrorMessage message={error.message} onRetry={refetch} />
  }

  if (!data || data.tracked_documents.length === 0) {
    return (
      <div className="flex h-full items-center justify-center p-8">
        <p className="text-muted-foreground text-sm">{t('followedDocs.empty')}</p>
      </div>
    )
  }

  return (
    <div className="space-y-3 p-6">
      <h1 className="text-xl font-semibold">{t('followedDocs.title')}</h1>
      <div className="space-y-2">
        {data.tracked_documents.map((doc) => (
          <DocumentListItem
            key={doc.uuid}
            document={doc}
            onClick={onSelectDocument}
          />
        ))}
      </div>
    </div>
  )
}
