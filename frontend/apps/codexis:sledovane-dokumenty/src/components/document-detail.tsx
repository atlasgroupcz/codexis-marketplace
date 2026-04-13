import { useEffect, useRef, useState } from 'react'
import { ArrowLeft, Info, Plus, Trash2, X } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Badge } from '@workspace/ui/components/badge'
import { useTranslation } from 'react-i18next'
import { TrackingTypeBadge } from './tracking-type-badge'
import { ChangeItem } from './change-item'
import { DetailSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { useDocumentDetail } from '@/hooks/use-document-detail'
import { postAction, postGroupAction, postNoteAction } from '@/lib/api'
import { formatDate } from '@/lib/format'

interface DocumentDetailProps {
  uuid: string
  onBack: () => void
}

export function DocumentDetail({ uuid, onBack }: DocumentDetailProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useDocumentDetail(uuid)
  const [confirmRemove, setConfirmRemove] = useState(false)
  const [removing, setRemoving] = useState(false)
  const confirmTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const [showGroupPicker, setShowGroupPicker] = useState(false)
  const [newGroupName, setNewGroupName] = useState('')
  const [newNote, setNewNote] = useState('')

  useEffect(() => {
    return () => {
      if (confirmTimerRef.current) clearTimeout(confirmTimerRef.current)
    }
  }, [])

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

  const handleRemove = (): void => {
    if (!confirmRemove) {
      setConfirmRemove(true)
      confirmTimerRef.current = setTimeout(() => setConfirmRemove(false), 3000)
      return
    }
    if (confirmTimerRef.current) clearTimeout(confirmTimerRef.current)
    setRemoving(true)
    postAction('remove', uuid)
      .then(() => onBack())
      .catch(() => setRemoving(false))
  }

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <Button variant="ghost" size="sm" onClick={onBack}>
          <ArrowLeft className="size-4" />
          {t('followedDocs.back')}
        </Button>
        <Button
          variant={confirmRemove ? 'destructive' : 'outline'}
          size="sm"
          onClick={handleRemove}
          disabled={removing}
          data-testid="remove-document"
        >
          <Trash2 className="size-4" />
          {confirmRemove ? t('followedDocs.removeConfirmButton') : t('followedDocs.removeTracking')}
        </Button>
      </div>

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

      {/* Group management */}
      <div className="space-y-2">
        <h2 className="text-sm font-medium">{t('followedDocs.groups')}</h2>
        <div className="flex flex-wrap items-center gap-1.5">
          {document.groups.map((g) => (
            <Badge key={g.id} variant="outline" className="gap-1 pr-1">
              {g.name}
              <button
                type="button"
                className="text-muted-foreground hover:text-destructive rounded-full p-0.5"
                onClick={() => {
                  postGroupAction('group_remove', {
                    codexisId: document.codexisId,
                    groupId: g.id,
                  }).then(() => refetch())
                }}
                title={t('followedDocs.removeFromGroup')}
              >
                <X className="size-3" />
              </button>
            </Badge>
          ))}
          <button
            type="button"
            className="text-muted-foreground hover:text-foreground rounded-full border border-dashed p-1 transition-colors"
            onClick={() => setShowGroupPicker(!showGroupPicker)}
            title={t('followedDocs.addToGroup')}
          >
            <Plus className="size-3" />
          </button>
        </div>
        {showGroupPicker && (
          <div className="space-y-2 rounded-lg border p-3">
            {/* Existing groups not yet assigned */}
            {data.groups
              .filter((g) => !document.groups.some((dg) => dg.id === g.id))
              .map((g) => (
                <button
                  key={g.id}
                  type="button"
                  className="hover:bg-accent block w-full rounded-md px-3 py-1.5 text-left text-sm transition-colors"
                  onClick={() => {
                    postGroupAction('group_add', {
                      codexisId: document.codexisId,
                      groupName: g.name,
                    }).then(() => {
                      setShowGroupPicker(false)
                      refetch()
                    })
                  }}
                >
                  {g.name}
                </button>
              ))}
            {/* Create new group */}
            <div className="flex items-center gap-2 border-t pt-2">
              <input
                type="text"
                className="border-input bg-background flex-1 rounded-md border px-3 py-1 text-sm"
                placeholder={t('followedDocs.groupName')}
                value={newGroupName}
                onChange={(e) => setNewGroupName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && newGroupName.trim()) {
                    postGroupAction('group_add', {
                      codexisId: document.codexisId,
                      groupName: newGroupName.trim(),
                    }).then(() => {
                      setNewGroupName('')
                      setShowGroupPicker(false)
                      refetch()
                    })
                  }
                }}
              />
              <Button
                size="sm"
                variant="outline"
                disabled={!newGroupName.trim()}
                onClick={() => {
                  if (!newGroupName.trim()) return
                  postGroupAction('group_add', {
                    codexisId: document.codexisId,
                    groupName: newGroupName.trim(),
                  }).then(() => {
                    setNewGroupName('')
                    setShowGroupPicker(false)
                    refetch()
                  })
                }}
              >
                {t('followedDocs.createGroup')}
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* Notes */}
      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <h2 className="text-sm font-medium">{t('followedDocs.notes')}</h2>
          <span className="text-muted-foreground flex items-center gap-1 text-xs">
            <Info className="size-3" />
            {t('followedDocs.notesHint')}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <input
            type="text"
            className="border-input bg-background flex-1 rounded-md border px-3 py-1 text-sm"
            placeholder={t('followedDocs.notePlaceholder')}
            value={newNote}
            onChange={(e) => setNewNote(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && newNote.trim()) {
                postNoteAction('note_add', uuid, { text: newNote.trim() }).then(() => {
                  setNewNote('')
                  refetch()
                })
              }
            }}
          />
          <Button
            size="sm"
            variant="outline"
            disabled={!newNote.trim()}
            onClick={() => {
              if (!newNote.trim()) return
              postNoteAction('note_add', uuid, { text: newNote.trim() }).then(() => {
                setNewNote('')
                refetch()
              })
            }}
          >
            {t('followedDocs.addNote')}
          </Button>
        </div>
        {document.user_notes.length > 0 && (
          <div className="space-y-1">
            {document.user_notes.map((note, i) => (
              <div key={i} className="bg-muted/50 flex items-start gap-2 rounded-md px-3 py-1.5 text-sm">
                <span className="flex-1">{note}</span>
                <button
                  type="button"
                  className="text-muted-foreground hover:text-destructive mt-0.5 shrink-0 rounded-full p-0.5"
                  onClick={() => {
                    postNoteAction('note_remove', uuid, { index: i }).then(() => refetch())
                  }}
                >
                  <X className="size-3" />
                </button>
              </div>
            ))}
          </div>
        )}
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
        {document.changes.length > 0 ? (
          document.changes.map((change, index) => (
            <ChangeItem
              key={index}
              change={change}
              changeIndex={index}
              uuid={uuid}
              onMutate={refetch}
            />
          ))
        ) : (
          <p className="text-muted-foreground text-sm">{t('followedDocs.noChanges')}</p>
        )}
      </div>
    </div>
  )
}
