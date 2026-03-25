import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ArrowLeft, Trash2, Plus, X } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Badge } from '@workspace/ui/components/badge'
import { useTopicDetail } from '@/hooks/use-topic-detail'
import { postAction } from '@/lib/api'
import { formatDate } from '@/lib/format'
import { LoadingSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'

interface TopicDetailProps {
  uuid: string
  onBack: () => void
  onSelectReport: (reportId: string) => void
}

export function TopicDetail({ uuid, onBack, onSelectReport }: TopicDetailProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useTopicDetail(uuid)
  const [noteText, setNoteText] = useState('')
  const [showRemoveConfirm, setShowRemoveConfirm] = useState(false)
  const [removing, setRemoving] = useState(false)

  if (loading) return <LoadingSkeleton />
  if (error) return <ErrorMessage error={error} onRetry={refetch} />
  if (!data) return null

  const topic = data.topic
  const reports = data.reports

  const handleDelete = () => {
    setRemoving(true)
    postAction({ action: 'delete', uuid })
      .then(() => onBack())
      .catch(() => setRemoving(false))
  }

  const handleAddNote = async () => {
    if (!noteText.trim()) return
    await postAction({ action: 'note_add', uuid, text: noteText.trim() })
    setNoteText('')
    refetch()
  }

  const handleRemoveNote = async (index: number) => {
    await postAction({ action: 'note_remove', uuid, index })
    refetch()
  }

  return (
    <div className="space-y-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <Button variant="ghost" size="sm" onClick={onBack}>
          <ArrowLeft className="size-4" />
          {t('judikatura.back')}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="text-muted-foreground hover:text-destructive"
          onClick={() => setShowRemoveConfirm(!showRemoveConfirm)}
        >
          <Trash2 className="size-4" />
          {t('judikatura.deleteTopic')}
        </Button>
      </div>

      {showRemoveConfirm && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-4 dark:border-red-900 dark:bg-red-950/50">
          <p className="text-sm font-medium">{t('judikatura.deleteConfirmTitle')}</p>
          <p className="mt-1 text-sm text-muted-foreground">
            {t('judikatura.deleteConfirmText')}
          </p>
          <div className="mt-3 flex gap-2">
            <Button
              variant="destructive"
              size="sm"
              disabled={removing}
              onClick={handleDelete}
            >
              {t('judikatura.confirm')}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowRemoveConfirm(false)}
            >
              {t('judikatura.cancel')}
            </Button>
          </div>
        </div>
      )}

      {/* Title */}
      <div>
        <h1 className="text-xl font-semibold">{topic.name}</h1>
        <div className="mt-1 flex items-center gap-2 text-sm text-muted-foreground">
          <span>{t('judikatura.trackedSince')}: {topic.created_at ? formatDate(topic.created_at) : '-'}</span>
          {topic.last_check_at && (
            <>
              <span>&middot;</span>
              <span>{t('judikatura.lastCheck')}: {formatDate(topic.last_check_at)}</span>
            </>
          )}
        </div>
      </div>

      {/* Areas */}
      {topic.areas.length > 0 && (
        <div>
          <h2 className="mb-3 text-sm font-semibold">{t('judikatura.areas')}</h2>
          <div className="space-y-2">
            {topic.areas.map((area) => (
              <div key={area.name} className="rounded-lg border-l-4 border-l-primary/40 bg-muted/30 px-4 py-3">
                <div className="font-medium text-sm">{area.name}</div>
                {area.baseline_summary && (
                  <p className="mt-1 text-xs text-muted-foreground line-clamp-2">
                    {area.baseline_summary}
                  </p>
                )}
                {!area.baseline_summary && (
                  <p className="mt-1 text-xs italic text-muted-foreground/60">
                    {t('judikatura.baseline')}: -
                  </p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Notes */}
      <div>
        <h2 className="mb-3 text-sm font-semibold">{t('judikatura.notes')}</h2>
        {topic.notes.length > 0 && (
          <div className="mb-3 flex flex-wrap gap-2">
            {topic.notes.map((note, i) => (
              <span
                key={i}
                className="inline-flex items-center gap-1 rounded-full border bg-card px-3 py-1 text-xs shadow-sm"
              >
                {note}
                <button
                  type="button"
                  className="ml-1 text-muted-foreground hover:text-destructive"
                  onClick={() => void handleRemoveNote(i)}
                >
                  <X className="size-3" />
                </button>
              </span>
            ))}
          </div>
        )}
        <div className="flex gap-2">
          <input
            type="text"
            value={noteText}
            onChange={(e) => setNoteText(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && void handleAddNote()}
            placeholder={t('judikatura.notePlaceholder')}
            className="flex-1 rounded-md border bg-background px-3 py-1.5 text-sm"
          />
          <Button variant="outline" size="sm" onClick={() => void handleAddNote()}>
            <Plus className="size-4" />
            {t('judikatura.addNote')}
          </Button>
        </div>
      </div>

      {/* Reports */}
      <div>
        <h2 className="mb-3 text-sm font-semibold">{t('judikatura.reports')}</h2>
        {reports.length === 0 ? (
          <p className="py-8 text-center text-sm text-muted-foreground">
            {t('judikatura.noReports')}
          </p>
        ) : (
          <div className="space-y-2">
            {[...reports].reverse().map((report) => (
              <button
                key={report.report_id}
                type="button"
                className="bg-card text-card-foreground flex w-full items-center gap-4 rounded-xl border p-4 text-left shadow-sm transition-colors hover:bg-accent/50"
                onClick={() => onSelectReport(report.report_id)}
              >
                <div className="flex-1 space-y-1">
                  <div className="font-semibold">
                    {report.checked_at ? formatDate(report.checked_at) : report.report_id}
                  </div>
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <span>{t('judikatura.foundCount', { count: report.found_count })}</span>
                    {report.period_from && report.period_to && (
                      <>
                        <span>&middot;</span>
                        <span>{formatDate(report.period_from)} – {formatDate(report.period_to)}</span>
                      </>
                    )}
                  </div>
                </div>
                {report.confirmed_on ? (
                  <Badge variant="outline">{t('judikatura.confirmed')}</Badge>
                ) : (
                  <Badge variant="destructive">{t('judikatura.notConfirmed')}</Badge>
                )}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
