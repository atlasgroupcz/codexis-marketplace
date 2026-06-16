import { useState } from 'react'
import { useQueryState } from 'nuqs'
import { useTranslation } from 'react-i18next'
import { Check, Plus, Trash2 } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from '@workspace/ui/components/input-group'
import { DocumentCard } from './document-card'
import type { Group, TrackedDocumentSummary } from '@/lib/schemas'
import { postGroupAction } from '@/lib/api'
import { groupParser } from '@/lib/url-state'

interface DocumentGridProps {
  documents: Array<TrackedDocumentSummary>
  groups: Array<Group>
  onSelectDocument: (uuid: string) => void
  onMutate: () => void
}

export function DocumentGrid({ documents, groups, onSelectDocument, onMutate }: DocumentGridProps) {
  const { t } = useTranslation()
  const [selectedGroup, setSelectedGroup] = useQueryState('group', groupParser)
  const [showCreateGroup, setShowCreateGroup] = useState(false)
  const [newGroupName, setNewGroupName] = useState('')
  const [creating, setCreating] = useState(false)
  const [manage, setManage] = useState(false)

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
        setShowCreateGroup(false)
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
        {manage ? (
          <>
            <span className="text-muted-foreground mr-1 text-xs">
              {t('followedDocs.deleteGroupHint')}
            </span>
            {groups.map((group) => (
              <Button
                key={group.id}
                size="sm"
                variant="outline"
                className="border-destructive/40 text-destructive hover:bg-destructive/10 rounded-full"
                onClick={() => handleDeleteGroup(group.id)}
              >
                <Trash2 />
                {group.name}
              </Button>
            ))}
            <Button size="sm" variant="ghost" onClick={() => setManage(false)}>
              {t('followedDocs.done')}
            </Button>
          </>
        ) : (
          <>
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

            {showCreateGroup ? (
              <InputGroup className="h-8 w-56 rounded-full">
                <InputGroupInput
                  autoFocus
                  value={newGroupName}
                  onChange={(e) => setNewGroupName(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleCreateGroup()}
                  placeholder={t('followedDocs.groupName')}
                />
                <InputGroupAddon align="inline-end">
                  <InputGroupButton
                    size="icon-xs"
                    disabled={creating || !newGroupName.trim()}
                    onClick={handleCreateGroup}
                    aria-label={t('followedDocs.createGroup')}
                  >
                    <Check />
                  </InputGroupButton>
                </InputGroupAddon>
              </InputGroup>
            ) : (
              <Button
                size="sm"
                variant="ghost"
                className="text-muted-foreground rounded-full border border-dashed"
                onClick={() => setShowCreateGroup(true)}
              >
                <Plus />
                {t('followedDocs.createGroup')}
              </Button>
            )}

            {groups.length > 0 && (
              <Button
                size="sm"
                variant="ghost"
                className="text-muted-foreground"
                onClick={() => setManage(true)}
                title={t('followedDocs.deleteGroup')}
              >
                <Trash2 />
              </Button>
            )}
          </>
        )}
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
