import { useQueryState } from 'nuqs'
import { useTranslation } from 'react-i18next'
import { ArrowRight, Plus, RefreshCw, Scale } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { Toaster } from '@workspace/ui/components/sonner'
import { AppHeader, HeaderGear, HeaderHelp } from '@workspace/ui/components/app-header'
import { TopicRail } from './topic-rail'
import { ReportsFeed } from './reports-feed'
import { EmptyTopics } from './empty-state'
import { FeedSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { useOverview } from '@/hooks/use-overview'
import { topicFilterParser } from '@/lib/url-state'

interface JudikaturaHomeProps {
  onSelectReport: (topicUuid: string, reportId: string) => void
  onOpenTopic: (uuid: string) => void
  onOpenSettings: () => void
}

export function JudikaturaHome({ onSelectReport, onOpenTopic, onOpenSettings }: JudikaturaHomeProps) {
  const { t, i18n } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const [topicFilter, setTopicFilter] = useQueryState('topic', topicFilterParser)

  const topics = data?.topics ?? []
  const activeTopic = topicFilter ? topics.find((topic) => topic.uuid === topicFilter) : undefined

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
      <AppHeader
        brand={{ icon: Scale, title: t('judikatura.title'), subtitle: t('judikatura.subtitle') }}
        actions={
          <>
            {topics.length > 0 && (
              <>
                <Button asChild size="sm" data-testid="add-monitoring">
                  <a
                    href={`/?draft=${encodeURIComponent(t('judikatura.landing.chatPrompt'))}`}
                    target="_top"
                  >
                    <Plus className="size-4" />
                    <span className="hidden sm:inline">{t('judikatura.addMonitoring')}</span>
                  </a>
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => void refetch()}
                  disabled={loading}
                >
                  <RefreshCw className={`size-4 ${loading ? 'animate-spin' : ''}`} />
                  <span className="hidden sm:inline">{t('judikatura.refresh')}</span>
                </Button>
              </>
            )}
            <HeaderHelp app="sledovana-judikatura" lang={i18n.language} title={t('help.docs')} />
            <HeaderGear onClick={onOpenSettings} title={t('judikatura.notify.title')} />
          </>
        }
      />

      <main className="min-h-0 flex-1">
        {loading && !data ? (
          <div className="mx-auto max-w-5xl p-4 sm:p-6">
            <FeedSkeleton />
          </div>
        ) : error ? (
          <div className="p-6">
            <ErrorMessage error={error} onRetry={refetch} />
          </div>
        ) : topics.length === 0 ? (
          <EmptyTopics />
        ) : (
          <div className="grid h-full min-h-0 md:grid-cols-[240px_1fr]">
            <aside className="hidden min-h-0 border-r md:block">
              <ScrollArea className="h-full">
                <div className="p-3">
                  <TopicRail
                    topics={topics}
                    topicFilter={topicFilter}
                    onSelectFilter={(uuid) => void setTopicFilter(uuid)}
                  />
                </div>
              </ScrollArea>
            </aside>
            <ScrollArea className="h-full min-h-0">
              <div className="mx-auto max-w-3xl space-y-5 p-4 sm:p-6">
                {activeTopic && (
                  <div className="bg-secondary/40 flex items-center justify-between gap-3 rounded-lg px-4 py-3">
                    <div className="min-w-0">
                      <p className="font-display truncate text-base">{activeTopic.name}</p>
                      <p className="text-muted-foreground text-xs">
                        {t('judikatura.areasCount', { count: activeTopic.areas })}
                      </p>
                    </div>
                    <Button variant="outline" size="sm" onClick={() => onOpenTopic(activeTopic.uuid)}>
                      {t('judikatura.topicDetail')}
                      <ArrowRight className="size-4" />
                    </Button>
                  </div>
                )}

                <ReportsFeed
                  topics={topics}
                  topicFilter={topicFilter}
                  onSelectReport={onSelectReport}
                />
              </div>
            </ScrollArea>
          </div>
        )}
      </main>

      <Toaster position="bottom-right" />
    </div>
  )
}
