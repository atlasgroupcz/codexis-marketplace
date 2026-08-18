import { AppHeader, HeaderGear } from '@workspace/ui/components/app-header'
import { Button } from '@workspace/ui/components/button'
import { Card, CardDescription, CardHeader, CardTitle } from '@workspace/ui/components/card'
import { ItemGroup } from '@workspace/ui/components/item'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { Textarea } from '@workspace/ui/components/textarea'
import { useConfirmAction } from '@workspace/ui/hooks/use-confirm-action'
import { FileText, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useTopicDetail } from '@/hooks/use-topic-detail'
import { postAction } from '@/lib/api'
import { formatDate, formatRelative } from '@/lib/format'
import { NoReports } from './empty-state'
import { ErrorMessage } from './error-message'
import { FeedItem } from './feed-item'
import { LoadingSkeleton } from './loading-skeleton'

interface TopicDetailProps {
  uuid: string
  onBack: () => void
  onSelectReport: (reportId: string) => void
  onOpenSettings: () => void
}

export function TopicDetail({ uuid, onBack, onSelectReport, onOpenSettings }: TopicDetailProps) {
  const { t, i18n } = useTranslation()
  const { data, loading, error, refetch } = useTopicDetail(uuid)
  const [noteDraft, setNoteDraft] = useState<string | null>(null)
  const [savingNotes, setSavingNotes] = useState(false)
  const remove = useConfirmAction(() =>
    postAction({ action: 'delete', uuid })
      .then(() => {
        toast.success(t('judikatura.removed'))
        onBack()
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e))),
  )

  if (loading && !data) return <LoadingSkeleton />
  if (error) {
    return (
      <div className="p-6">
        <ErrorMessage error={error} onRetry={refetch} />
      </div>
    )
  }
  if (!data) return null

  const topic = data.topic
  const notesText = topic.notes.join('\n')
  const noteValue = noteDraft ?? notesText
  const notesDirty = noteValue !== notesText
  const reports = [...data.reports].sort((a, b) =>
    (a.checked_at ?? '') < (b.checked_at ?? '') ? 1 : -1,
  )

  const handleSaveNotes = async () => {
    setSavingNotes(true)
    try {
      for (let i = topic.notes.length - 1; i >= 0; i--) {
        await postAction({ action: 'note_remove', uuid, index: i })
      }
      const text = noteValue.trim()
      if (text) await postAction({ action: 'note_add', uuid, text })
      setNoteDraft(null)
      toast.success(t('judikatura.noteSaved'))
      refetch()
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e))
    } finally {
      setSavingNotes(false)
    }
  }

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
      <AppHeader
        back={{
          onClick: onBack,
          label: t('judikatura.back'),
          title: topic.name,
          subtitle: topic.last_check_at
            ? `${t('judikatura.lastCheck')}: ${formatDate(topic.last_check_at)}`
            : t('judikatura.areasCount', { count: topic.areas.length }),
        }}
        docs={{ app: 'sledovana-judikatura', lang: i18n.language, label: t('help.docs') }}
        actions={
          <>
            <Button
              variant={remove.confirming ? 'destructive' : 'outline'}
              size="sm"
              onClick={remove.trigger}
              disabled={remove.pending}
              data-testid="delete-topic"
            >
              <Trash2 className="size-4" />
              <span className="hidden sm:inline">
                {remove.confirming
                  ? t('judikatura.deleteConfirmTitle')
                  : t('judikatura.deleteTopic')}
              </span>
            </Button>
            <HeaderGear onClick={onOpenSettings} title={t('judikatura.notify.title')} />
          </>
        }
      />

      <main className="min-h-0 flex-1">
        <ScrollArea className="h-full">
          <div className="mx-auto max-w-3xl space-y-6 p-4 sm:p-6">
            {topic.areas.length > 0 && (
              <section className="space-y-2">
                <h2 className="text-sm font-medium">{t('judikatura.areas')}</h2>
                <div className="space-y-3">
                  {topic.areas.map((area) => (
                    <Card key={area.name} className="gap-2 py-4">
                      <CardHeader className="gap-1 px-4">
                        <CardTitle className="text-sm">{area.name}</CardTitle>
                        <CardDescription className="leading-relaxed whitespace-pre-line">
                          {area.baseline_summary ?? `${t('judikatura.baseline')}: -`}
                        </CardDescription>
                      </CardHeader>
                    </Card>
                  ))}
                </div>
              </section>
            )}

            <section className="space-y-2">
              <h2 className="text-sm font-medium">{t('judikatura.notes')}</h2>
              <Textarea
                value={noteValue}
                onChange={(e) => setNoteDraft(e.target.value)}
                placeholder={t('judikatura.notePlaceholder')}
                className="min-h-24"
              />
              <div className="flex justify-end">
                <Button
                  size="sm"
                  onClick={() => void handleSaveNotes()}
                  disabled={!notesDirty || savingNotes}
                >
                  {t('judikatura.saveNote')}
                </Button>
              </div>
            </section>

            <section className="space-y-3">
              <h2 className="font-display text-lg">{t('judikatura.reports')}</h2>
              {reports.length === 0 ? (
                <NoReports />
              ) : (
                <ItemGroup className="gap-1.5">
                  {reports.map((report) => {
                    const unread = !report.confirmed_on
                    const period =
                      report.period_from && report.period_to
                        ? ` · ${formatDate(report.period_from)} - ${formatDate(report.period_to)}`
                        : ''
                    return (
                      <FeedItem
                        key={report.report_id}
                        unread={unread}
                        icon={
                          <FileText className={unread ? 'text-primary' : 'text-muted-foreground'} />
                        }
                        title={t('judikatura.report')}
                        description={`${t('judikatura.foundCount', { count: report.found_count })}${period}`}
                        date={
                          report.checked_at ? formatRelative(report.checked_at) : report.report_id
                        }
                        dateTitle={report.checked_at ? formatDate(report.checked_at) : undefined}
                        onClick={() => onSelectReport(report.report_id)}
                      />
                    )
                  })}
                </ItemGroup>
              )}
            </section>
          </div>
        </ScrollArea>
      </main>

      <Toaster position="bottom-right" />
    </div>
  )
}
