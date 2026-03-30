import { useTranslation } from 'react-i18next'
import { Info } from 'lucide-react'
import { Badge } from '@workspace/ui/components/badge'
import { useOverview } from '@/hooks/use-overview'
import { LoadingSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { formatDate } from '@/lib/format'

interface TopicListProps {
  onSelectTopic: (uuid: string) => void
}

export function TopicList({ onSelectTopic }: TopicListProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()

  if (loading && !data) return <LoadingSkeleton />
  if (error) return <ErrorMessage error={error} onRetry={refetch} />
  if (!data) return null

  const topics = data.topics

  return (
    <div className="space-y-4 p-6">
      <h1 className="text-xl font-semibold">{t('judikatura.title')}</h1>

      {topics.length === 0 ? (
        <div className="flex gap-3 rounded-lg border border-dashed border-blue-300 bg-blue-50 p-4 text-blue-900 dark:border-blue-800 dark:bg-blue-950/50 dark:text-blue-200">
          <Info className="mt-0.5 size-4 shrink-0" />
          <div className="text-sm">
            <p>{t('judikatura.empty')}</p>
          </div>
        </div>
      ) : (
        <div className="space-y-2">
          {topics.map((topic) => (
            <button
              key={topic.uuid}
              type="button"
              className="bg-card text-card-foreground flex w-full items-center gap-4 rounded-xl border p-4 text-left shadow-sm transition-colors hover:bg-accent/50"
              onClick={() => onSelectTopic(topic.uuid)}
            >
              <div className="flex-1 space-y-1">
                <div className="font-semibold">{topic.name}</div>
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <span>{t('judikatura.areasCount', { count: topic.areas })}</span>
                  <span>&middot;</span>
                  <span>{t('judikatura.reportsCount', { count: topic.total_reports })}</span>
                  {topic.last_check_at && (
                    <>
                      <span>&middot;</span>
                      <span>{t('judikatura.lastCheck')}: {formatDate(topic.last_check_at)}</span>
                    </>
                  )}
                </div>
              </div>
              {topic.unconfirmed_reports > 0 && (
                <Badge variant="destructive">
                  {t('judikatura.unconfirmedReports', { count: topic.unconfirmed_reports })}
                </Badge>
              )}
              <Badge variant="secondary">
                {t('judikatura.reportsCount', { count: topic.total_reports })}
              </Badge>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
