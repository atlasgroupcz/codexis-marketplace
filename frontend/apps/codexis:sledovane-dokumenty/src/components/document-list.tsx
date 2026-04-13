import { useState } from 'react'
import { useQueryState } from 'nuqs'
import { Info, Plus, X } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { useTranslation } from 'react-i18next'
import { DocumentListItem } from './document-list-item'
import { ListSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { useOverview } from '@/hooks/use-overview'
import { postGroupAction } from '@/lib/api'
import { groupParser } from '@/lib/url-state'
import type { TrackedDocumentSummary } from '@/lib/schemas'

interface DocumentListProps {
  onSelectDocument: (uuid: string) => void
}

function AddHint(): React.JSX.Element {
  const { t } = useTranslation()
  return (
    <div className="flex gap-3 rounded-lg border border-dashed border-blue-300 bg-blue-50 p-4 text-blue-900 dark:border-blue-800 dark:bg-blue-950/50 dark:text-blue-200">
      <Info className="mt-0.5 size-4 shrink-0" />
      <div className="text-sm">
        <p className="font-medium">{t('followedDocs.addHintTitle')}</p>
        <p className="mt-1">{t('followedDocs.addHintText')}</p>
      </div>
    </div>
  )
}

export function DocumentList({ onSelectDocument }: DocumentListProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const [selectedGroup, setSelectedGroup] = useQueryState('group', groupParser)
  const [showCreateGroup, setShowCreateGroup] = useState(false)
  const [newGroupName, setNewGroupName] = useState('')
  const [creating, setCreating] = useState(false)

  if (loading && !data) {
    return <ListSkeleton />
  }

  if (error) {
    return <ErrorMessage message={error.message} onRetry={refetch} />
  }

  if (!data || data.tracked_documents.length === 0) {
    return (
      <div className="space-y-6 p-6">
        <AddHint />
        <h1 className="text-xl font-semibold">{t('followedDocs.title')}</h1>
        <p className="text-muted-foreground text-sm">{t('followedDocs.empty')}</p>
      </div>
    )
  }

  const groups = data.groups ?? []

  const filterDocuments = (docs: TrackedDocumentSummary[]): TrackedDocumentSummary[] => {
    if (!selectedGroup) return docs
    if (selectedGroup === '__ungrouped__') {
      return docs.filter((doc) => !doc.groups || doc.groups.length === 0)
    }
    return docs.filter((doc) =>
      doc.groups?.some((g) => g.id === selectedGroup),
    )
  }

  const filteredDocs = filterDocuments(data.tracked_documents)
  const ungroupedCount = data.tracked_documents.filter(
    (doc) => !doc.groups || doc.groups.length === 0,
  ).length

  const handleCreateGroup = () => {
    if (!newGroupName.trim()) return
    setCreating(true)
    postGroupAction('group_add', {
      groupName: newGroupName.trim(),
    })
      .then(() => {
        setNewGroupName('')
        setShowCreateGroup(false)
        refetch()
      })
      .finally(() => setCreating(false))
  }

  const handleDeleteGroup = (groupId: string) => {
    postGroupAction('group_delete', { groupId }).then(() => {
      if (selectedGroup === groupId) setSelectedGroup(null)
      refetch()
    })
  }

  return (
    <div className="space-y-4 p-6">
      <AddHint />
      <h1 className="text-xl font-semibold">{t('followedDocs.title')}</h1>

      {/* Group filter tabs */}
      {(groups.length > 0 || showCreateGroup) && (
        <div className="space-y-2">
          <div className="flex flex-wrap items-center gap-1.5">
            <button
              type="button"
              className={`rounded-full border px-3 py-1 text-xs font-medium transition-colors ${
                !selectedGroup
                  ? 'border-primary bg-primary text-primary-foreground'
                  : 'border-border hover:bg-accent'
              }`}
              onClick={() => setSelectedGroup(null)}
            >
              {t('followedDocs.allDocuments')}
            </button>
            {groups.map((group) => (
              <div key={group.id} className="group relative flex items-center">
                <button
                  type="button"
                  className={`rounded-full border px-3 py-1 text-xs font-medium transition-colors ${
                    selectedGroup === group.id
                      ? 'border-primary bg-primary text-primary-foreground'
                      : 'border-border hover:bg-accent'
                  }`}
                  onClick={() =>
                    setSelectedGroup(selectedGroup === group.id ? null : group.id)
                  }
                >
                  {group.name}
                  <span className="ml-1 opacity-60">({group.members.length})</span>
                </button>
                <button
                  type="button"
                  className="text-muted-foreground hover:text-destructive ml-0.5 hidden rounded-full p-0.5 group-hover:inline-flex"
                  onClick={(e) => {
                    e.stopPropagation()
                    handleDeleteGroup(group.id)
                  }}
                  title={t('followedDocs.deleteGroup')}
                >
                  <X className="size-3" />
                </button>
              </div>
            ))}
            {ungroupedCount > 0 && groups.length > 0 && (
              <button
                type="button"
                className={`rounded-full border px-3 py-1 text-xs font-medium transition-colors ${
                  selectedGroup === '__ungrouped__'
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border hover:bg-accent'
                }`}
                onClick={() =>
                  setSelectedGroup(
                    selectedGroup === '__ungrouped__' ? null : '__ungrouped__',
                  )
                }
              >
                {t('followedDocs.ungrouped')}
                <span className="ml-1 opacity-60">({ungroupedCount})</span>
              </button>
            )}
            <button
              type="button"
              className="text-muted-foreground hover:text-foreground rounded-full border border-dashed p-1 transition-colors"
              onClick={() => setShowCreateGroup(!showCreateGroup)}
              title={t('followedDocs.createGroup')}
            >
              <Plus className="size-3" />
            </button>
          </div>

          {showCreateGroup && (
            <div className="flex items-center gap-2">
              <input
                type="text"
                className="border-input bg-background rounded-md border px-3 py-1 text-sm"
                placeholder={t('followedDocs.groupName')}
                value={newGroupName}
                onChange={(e) => setNewGroupName(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleCreateGroup()}
              />
              <Button
                size="sm"
                variant="outline"
                disabled={creating || !newGroupName.trim()}
                onClick={handleCreateGroup}
              >
                {t('followedDocs.createGroup')}
              </Button>
            </div>
          )}
        </div>
      )}

      <div className="space-y-2">
        {filteredDocs.map((doc) => (
          <DocumentListItem
            key={doc.uuid}
            document={doc}
            onClick={onSelectDocument}
          />
        ))}
        {filteredDocs.length === 0 && (
          <p className="text-muted-foreground text-sm">{t('followedDocs.empty')}</p>
        )}
      </div>
    </div>
  )
}
