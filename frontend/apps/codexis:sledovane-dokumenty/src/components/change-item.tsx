import Markdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Badge } from '@workspace/ui/components/badge'
import { useTranslation } from 'react-i18next'
import { ChangeTypeBadge } from './change-type-badge'
import type { Change } from '@/lib/schemas'
import { formatDate, formatDateTime } from '@/lib/format'

interface ChangeItemProps {
  change: Change
}

export function ChangeItem({ change }: ChangeItemProps) {
  const { t } = useTranslation()

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
          <Badge variant="destructive">{t('followedDocs.notConfirmed')}</Badge>
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

      {change.source_documents.length > 0 && (
        <div className="text-sm">
          <span className="text-muted-foreground">
            {t('followedDocs.sourceDocuments')}{' '}
          </span>
          {change.source_documents.map((doc, i) => (
            <span key={doc.codexisId}>
              {i > 0 && ', '}
              <span className="font-medium">{doc.name}</span>
              <span className="text-muted-foreground"> ({doc.codexisId})</span>
            </span>
          ))}
        </div>
      )}

      <div className="prose prose-sm dark:prose-invert max-w-none">
        <Markdown remarkPlugins={[remarkGfm]}>
          {change.description_md}
        </Markdown>
      </div>
    </div>
  )
}
