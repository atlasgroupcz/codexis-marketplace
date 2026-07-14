import { useState } from 'react'
import { ArrowLeft, BookMarked, FileText, Hash, Info, Link2, Plus, Trash2, X } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Input } from '@workspace/ui/components/input'
import { Textarea } from '@workspace/ui/components/textarea'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { HeaderGear } from '@workspace/ui/components/app-header'
import { cn } from '@workspace/ui/lib/utils'
import { useConfirmAction } from '@workspace/ui/hooks/use-confirm-action'
import { useTranslation } from 'react-i18next'
import { TrackingTypeBadge } from './tracking-type-badge'
import { ChangeItem } from './change-item'
import { NoChanges } from './empty-state'
import { DetailSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { useDocumentDetail } from '@/hooks/use-document-detail'
import { postAction, postGroupAction, postNoteAction } from '@/lib/api'
import { formatDate } from '@/lib/format'

interface DocumentDetailProps {
  uuid: string
  onBack: () => void
  onOpenSettings: () => void
}

export function DocumentDetail({ uuid, onBack, onOpenSettings }: DocumentDetailProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useDocumentDetail(uuid)
  const remove = useConfirmAction(() =>
    postAction('remove', uuid)
      .then(() => {
        toast.success(t('followedDocs.removed'))
        onBack()
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e))),
  )
  const [showGroupPicker, setShowGroupPicker] = useState(false)
  const [newGroupName, setNewGroupName] = useState('')
  const [noteDraft, setNoteDraft] = useState<string | null>(null)
  const [savingNotes, setSavingNotes] = useState(false)

  if (loading && !data) {
    return <DetailSkeleton />
  }
  if (error) {
    return (
      <div className="p-6">
        <ErrorMessage message={error.message} onRetry={refetch} />
      </div>
    )
  }
  if (!data) {
    return null
  }

  const { document } = data
  const notesText = document.user_notes.join('\n')
  const noteValue = noteDraft ?? notesText
  const notesDirty = noteValue !== notesText
  const ordered = [...document.changes].sort((a, b) => (a.detected_on < b.detected_on ? 1 : -1))

  const handleSaveNotes = async (): Promise<void> => {
    setSavingNotes(true)
    try {
      for (let i = document.user_notes.length - 1; i >= 0; i--) {
        await postNoteAction('note_remove', uuid, { index: i })
      }
      const text = noteValue.trim()
      if (text) await postNoteAction('note_add', uuid, { text })
      setNoteDraft(null)
      toast.success(t('followedDocs.noteSaved'))
      refetch()
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e))
    } finally {
      setSavingNotes(false)
    }
  }

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
      <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b px-4">
        <div className="flex min-w-0 items-center gap-2">
          <Button variant="ghost" size="sm" onClick={onBack} className="-ml-2 shrink-0">
            <ArrowLeft className="size-4" />
            <span className="hidden sm:inline">{t('followedDocs.back')}</span>
          </Button>
          <span className="bg-secondary text-primary ring-brand-100 hidden size-9 shrink-0 items-center justify-center rounded-xl ring-1 sm:flex">
            <BookMarked className="size-5" strokeWidth={1.75} />
          </span>
          <div className="min-w-0">
            <h1 className="font-display text-foreground truncate text-[15px] leading-tight">
              {document.name}
            </h1>
            <p className="text-muted-foreground hidden truncate text-xs sm:block">
              {t('followedDocs.trackedSince', { date: formatDate(document.added_on) })}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant={remove.confirming ? 'destructive' : 'outline'}
            size="sm"
            onClick={remove.trigger}
            disabled={remove.pending}
            data-testid="remove-document"
          >
            <Trash2 className="size-4" />
            <span className="hidden sm:inline">
              {remove.confirming ? t('followedDocs.removeConfirmButton') : t('followedDocs.removeTracking')}
            </span>
          </Button>
          <HeaderGear onClick={onOpenSettings} title={t('notify.title')} />
        </div>
      </header>

      <main className="min-h-0 flex-1">
        <ScrollArea className="h-full">
          <div className="mx-auto max-w-3xl space-y-6 p-4 sm:p-6">
            <div className="text-muted-foreground flex flex-wrap items-center gap-2 text-sm">
              <TrackingTypeBadge type={document.tracking_type} />
              <span>{t('followedDocs.total', { count: document.total_changes })}</span>
              {document.unconfirmed_changes > 0 && (
                <>
                  <span className="text-border">·</span>
                  <span className="text-primary font-medium">
                    {t('followedDocs.unconfirmed', { count: document.unconfirmed_changes })}
                  </span>
                </>
              )}
            </div>

            {/* Groups */}
            <section className="space-y-2">
              <h2 className="text-sm font-medium">{t('followedDocs.groups')}</h2>
              <div className="flex flex-wrap items-center gap-2">
                {document.groups.map((g) => (
                  <span
                    key={g.id}
                    className="border-input bg-background inline-flex h-8 items-center gap-1 rounded-full border pr-1.5 pl-3 text-sm"
                  >
                    {g.name}
                    <button
                      type="button"
                      className="text-muted-foreground hover:text-destructive rounded-full p-1"
                      onClick={() =>
                        void postGroupAction('group_remove', {
                          codexisId: document.codexisId,
                          groupId: g.id,
                        }).then(refetch)
                      }
                      title={t('followedDocs.removeFromGroup')}
                    >
                      <X className="size-3.5" />
                    </button>
                  </span>
                ))}
                <Button
                  variant="ghost"
                  size="sm"
                  className="text-muted-foreground rounded-full border border-dashed"
                  onClick={() => setShowGroupPicker(!showGroupPicker)}
                >
                  <Plus className="size-3" />
                  {t('followedDocs.addToGroup')}
                </Button>
              </div>
              {showGroupPicker && (
                <div className="space-y-2 rounded-lg border p-3">
                  {data.groups
                    .filter((g) => !document.groups.some((dg) => dg.id === g.id))
                    .map((g) => (
                      <button
                        key={g.id}
                        type="button"
                        className="hover:bg-accent block w-full rounded-md px-3 py-1.5 text-left text-sm transition-colors"
                        onClick={() =>
                          void postGroupAction('group_add', {
                            codexisId: document.codexisId,
                            groupName: g.name,
                          }).then(() => {
                            setShowGroupPicker(false)
                            refetch()
                          })
                        }
                      >
                        {g.name}
                      </button>
                    ))}
                  <div className="flex items-center gap-2 border-t pt-2">
                    <Input
                      value={newGroupName}
                      onChange={(e) => setNewGroupName(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' && newGroupName.trim()) {
                          void postGroupAction('group_add', {
                            codexisId: document.codexisId,
                            groupName: newGroupName.trim(),
                          }).then(() => {
                            setNewGroupName('')
                            setShowGroupPicker(false)
                            refetch()
                          })
                        }
                      }}
                      placeholder={t('followedDocs.groupName')}
                      className="h-8 flex-1"
                    />
                  </div>
                </div>
              )}
            </section>

            {/* Notes — one continuous note */}
            <section className="space-y-2">
              <div className="flex items-center gap-2">
                <h2 className="text-sm font-medium">{t('followedDocs.notes')}</h2>
                <span className="text-muted-foreground flex items-center gap-1 text-xs">
                  <Info className="size-3" />
                  {t('followedDocs.notesHint')}
                </span>
              </div>
              <Textarea
                value={noteValue}
                onChange={(e) => setNoteDraft(e.target.value)}
                placeholder={t('followedDocs.notePlaceholder')}
                className="min-h-24"
              />
              <div className="flex justify-end">
                <Button
                  size="sm"
                  onClick={() => void handleSaveNotes()}
                  disabled={!notesDirty || savingNotes}
                >
                  {t('followedDocs.saveNote')}
                </Button>
              </div>
            </section>

            {/* Tracked parts — stacked */}
            {document.parts.length > 0 && (
              <section className="space-y-2">
                <h2 className="text-sm font-medium">{t('followedDocs.trackedParts')}</h2>
                <div className="space-y-1">
                  {document.parts.map((part) => (
                    <div
                      key={part.partId}
                      className="flex items-center gap-2 rounded-md border px-3 py-2 text-sm"
                    >
                      <Hash className="text-muted-foreground size-3.5 shrink-0" />
                      {part.label}
                    </div>
                  ))}
                </div>
              </section>
            )}

            {/* Change timeline — type under the icon, line through icon centres */}
            <section className="space-y-3">
              <h2 className="font-display text-lg">
                {t('followedDocs.changes', { count: document.changes.length })}
              </h2>
              {ordered.length > 0 ? (
                <ol>
                  {ordered.map((change, index) => {
                    const related = change.change_type === 'related_change'
                    const Icon = related ? Link2 : FileText
                    const isLast = index === ordered.length - 1
                    return (
                      <li
                        key={`${change.detected_on}-${index}`}
                        className="grid grid-cols-[5.5rem_1fr] gap-3"
                      >
                        <div className="flex flex-col items-center pt-1">
                          <span
                            className={cn(
                              'bg-background flex size-9 items-center justify-center rounded-full border',
                              !change.confirmed_on && 'border-primary/40',
                            )}
                          >
                            <Icon
                              className={cn('size-4', related ? 'text-amber-600' : 'text-primary')}
                            />
                          </span>
                          <span
                            className={cn(
                              'mt-1.5 text-center text-[10px] leading-tight',
                              related ? 'text-amber-700' : 'text-primary',
                            )}
                          >
                            {related
                              ? t('followedDocs.changeType.relatedChange')
                              : t('followedDocs.changeType.documentChange')}
                          </span>
                          {!isLast && <span className="bg-border mt-2 w-px flex-1" />}
                        </div>
                        <div className="pb-6">
                          <ChangeItem
                            change={change}
                            changeIndex={document.changes.indexOf(change)}
                            uuid={uuid}
                            onMutate={refetch}
                          />
                        </div>
                      </li>
                    )
                  })}
                </ol>
              ) : (
                <NoChanges />
              )}
            </section>
          </div>
        </ScrollArea>
      </main>

      <Toaster position="bottom-right" />
    </div>
  )
}
