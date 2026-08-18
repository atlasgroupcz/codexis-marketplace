import { Button } from '@workspace/ui/components/button'
import { Input } from '@workspace/ui/components/input'
import { Popover, PopoverContent, PopoverTrigger } from '@workspace/ui/components/popover'
import { Plus, Settings2, Trash2 } from 'lucide-react'
import { useQueryState } from 'nuqs'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { postGroupAction } from '@/lib/api'
import type { Group, TrackedDocumentSummary } from '@/lib/schemas'
import { groupParser } from '@/lib/url-state'
import { DocumentCard } from './document-card'

interface DocumentGridProps {
  documents: Array<TrackedDocumentSummary>
  groups: Array<Group>
  onSelectDocument: (uuid: string) => void
  onMutate: () => void
}

export function DocumentGrid({ documents, groups, onSelectDocument, onMutate }: DocumentGridProps) {
  const { t } = useTranslation()
  const [selectedGroup, setSelectedGroup] = useQueryState('group', groupParser)
  const [newGroupName, setNewGroupName] = useState('')
  const [creating, setCreating] = useState(false)

  const ungroupedCount = documents.filter((doc) => doc.groups.length === 0).length
  const filtered = documents.filter((doc) => {
    if (!selectedGroup) return true
    if (selectedGroup === '__ungrouped__') return doc.groups.length === 0
    return doc.groups.some((g) => g.id === selectedGroup)
  })

  const handleCreateGroup = () => {
    if (!newGroupName.trim()) return
    setCreating(true)
    postGroupAction('group_add', { groupName: newGroupName.trim() })
      .then(() => {
        setNewGroupName('')
        onMutate()
      })
      .finally(() => setCreating(false))
  }

  const handleDeleteGroup = (groupId: string) => {
    void postGroupAction('group_delete', { groupId }).then(() => {
      if (selectedGroup === groupId) void setSelectedGroup(null)
      onMutate()
    })
  }

  const pill = (active: boolean): 'default' | 'outline' => (active ? 'default' : 'outline')

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          variant={pill(!selectedGroup)}
          className="rounded-full"
          onClick={() => setSelectedGroup(null)}
        >
          {t('followedDocs.allDocuments')}
          <span className="opacity-70">{documents.length}</span>
        </Button>

        {groups.map((group) => {
          const active = selectedGroup === group.id
          return (
            <Button
              key={group.id}
              size="sm"
              variant={pill(active)}
              className="rounded-full"
              onClick={() => setSelectedGroup(active ? null : group.id)}
            >
              {group.name}
              <span className="opacity-70">{group.members.length}</span>
            </Button>
          )
        })}

        {ungroupedCount > 0 && groups.length > 0 && (
          <Button
            size="sm"
            variant={pill(selectedGroup === '__ungrouped__')}
            className="rounded-full"
            onClick={() =>
              setSelectedGroup(selectedGroup === '__ungrouped__' ? null : '__ungrouped__')
            }
          >
            {t('followedDocs.ungrouped')}
            <span className="opacity-70">{ungroupedCount}</span>
          </Button>
        )}

        {/* Group management lives in a popover, so the filter row never shifts. */}
        <Popover>
          <PopoverTrigger asChild>
            <Button
              size="sm"
              variant="ghost"
              className="text-muted-foreground rounded-full border border-dashed"
            >
              <Settings2 />
              {t('followedDocs.manageGroups')}
            </Button>
          </PopoverTrigger>
          <PopoverContent align="start" className="w-72 space-y-3">
            <p className="text-sm font-medium">{t('followedDocs.manageGroups')}</p>
            <div className="flex items-center gap-2">
              <Input
                value={newGroupName}
                onChange={(e) => setNewGroupName(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleCreateGroup()}
                placeholder={t('followedDocs.groupName')}
                className="h-8"
              />
              <Button
                size="sm"
                variant="outline"
                disabled={creating || !newGroupName.trim()}
                onClick={handleCreateGroup}
              >
                <Plus />
                {t('followedDocs.createGroup')}
              </Button>
            </div>
            {groups.length > 0 ? (
              <ul className="space-y-1">
                {groups.map((group) => (
                  <li
                    key={group.id}
                    className="flex items-center justify-between gap-2 rounded-md px-2 py-1 text-sm"
                  >
                    <span className="truncate">
                      {group.name}
                      <span className="text-muted-foreground ml-1">({group.members.length})</span>
                    </span>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="text-muted-foreground hover:text-destructive size-7"
                      onClick={() => handleDeleteGroup(group.id)}
                      title={t('followedDocs.deleteGroup')}
                    >
                      <Trash2 />
                    </Button>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="text-muted-foreground text-xs">{t('followedDocs.noGroups')}</p>
            )}
          </PopoverContent>
        </Popover>
      </div>

      {filtered.length === 0 ? (
        <p className="text-muted-foreground py-8 text-center text-sm">{t('followedDocs.empty')}</p>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {filtered.map((doc) => (
            <DocumentCard key={doc.uuid} document={doc} onClick={onSelectDocument} />
          ))}
        </div>
      )}
    </div>
  )
}
