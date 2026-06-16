import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FileText, Search } from 'lucide-react'
import { Label } from '@workspace/ui/components/label'
import { Switch } from '@workspace/ui/components/switch'
import { ItemGroup } from '@workspace/ui/components/item'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from '@workspace/ui/components/input-group'
import { FeedItem } from './feed-item'
import { NoReports } from './empty-state'
import { FeedSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import type { TopicSummary } from '@/lib/schemas'
import { useReportsFeed } from '@/hooks/use-reports-feed'
import { formatDate, formatRelative } from '@/lib/format'

interface ReportsFeedProps {
  topics: Array<TopicSummary>
  /** When set, only reports of this topic are shown. */
  topicFilter: string | null
  onSelectReport: (topicUuid: string, reportId: string) => void
}

export function ReportsFeed({ topics, topicFilter, onSelectReport }: ReportsFeedProps) {
  const { t } = useTranslation()
  const { reports, loading, error, refetch } = useReportsFeed(topics)
  const [query, setQuery] = useState('')
  const [onlyUnconfirmed, setOnlyUnconfirmed] = useState(false)

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase()
    return reports.filter((report) => {
      if (topicFilter && report.topicUuid !== topicFilter) return false
      if (onlyUnconfirmed && report.confirmed_on) return false
      if (!needle) return true
      return report.topicName.toLowerCase().includes(needle)
    })
  }, [reports, topicFilter, query, onlyUnconfirmed])

  if (loading) return <FeedSkeleton />
  if (error) return <ErrorMessage error={error} onRetry={refetch} />
  if (reports.length === 0) return <NoReports />

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <InputGroup className="min-w-56 flex-1 rounded-md">
          <InputGroupAddon>
            <Search />
          </InputGroupAddon>
          <InputGroupInput
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t('judikatura.feed.searchPlaceholder')}
            aria-label={t('judikatura.feed.searchPlaceholder')}
          />
        </InputGroup>
        <Label className="text-muted-foreground flex items-center gap-2 text-sm font-normal">
          <Switch checked={onlyUnconfirmed} onCheckedChange={setOnlyUnconfirmed} />
          {t('judikatura.feed.onlyUnconfirmed')}
        </Label>
      </div>

      {filtered.length === 0 ? (
        <p className="text-muted-foreground py-8 text-center text-sm">
          {t('judikatura.feed.emptyFiltered')}
        </p>
      ) : (
        <ItemGroup className="gap-1.5">
          {filtered.map((report) => {
            const unread = !report.confirmed_on
            const period =
              report.period_from && report.period_to
                ? ` · ${formatDate(report.period_from)} – ${formatDate(report.period_to)}`
                : ''
            return (
              <FeedItem
                key={`${report.topicUuid}-${report.report_id}`}
                unread={unread}
                icon={<FileText className={unread ? 'text-primary' : 'text-muted-foreground'} />}
                title={report.topicName}
                description={`${t('judikatura.foundCount', { count: report.found_count })}${period}`}
                date={report.checked_at ? formatRelative(report.checked_at) : report.report_id}
                dateTitle={report.checked_at ? formatDate(report.checked_at) : undefined}
                onClick={() => onSelectReport(report.topicUuid, report.report_id)}
              />
            )
          })}
        </ItemGroup>
      )}
    </div>
  )
}
