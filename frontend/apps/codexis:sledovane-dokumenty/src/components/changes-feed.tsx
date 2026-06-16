import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FileText, Link2, Search } from 'lucide-react'
import { Label } from '@workspace/ui/components/label'
import { Switch } from '@workspace/ui/components/switch'
import { ItemGroup } from '@workspace/ui/components/item'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from '@workspace/ui/components/input-group'
import { FeedItem } from './feed-item'
import { NoChanges } from './empty-state'
import { FeedSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import type { TrackedDocumentSummary } from '@/lib/schemas'
import { useChangesFeed } from '@/hooks/use-changes-feed'
import { formatDateTime, formatRelative, plainSnippet } from '@/lib/format'

interface ChangesFeedProps {
  documents: Array<TrackedDocumentSummary>
  onSelectDocument: (uuid: string) => void
}

export function ChangesFeed({ documents, onSelectDocument }: ChangesFeedProps) {
  const { t } = useTranslation()
  const { changes, loading, error, refetch } = useChangesFeed(documents)
  const [query, setQuery] = useState('')
  const [onlyUnconfirmed, setOnlyUnconfirmed] = useState(false)

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase()
    return changes.filter((change) => {
      if (onlyUnconfirmed && change.confirmed_on) return false
      if (!needle) return true
      return (
        change.documentName.toLowerCase().includes(needle) ||
        change.description_md.toLowerCase().includes(needle)
      )
    })
  }, [changes, query, onlyUnconfirmed])

  if (loading) return <FeedSkeleton />
  if (error) return <ErrorMessage message={error.message} onRetry={refetch} />
  if (changes.length === 0) return <NoChanges />

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
            placeholder={t('followedDocs.feed.searchPlaceholder')}
            aria-label={t('followedDocs.feed.searchPlaceholder')}
          />
        </InputGroup>
        <Label className="text-muted-foreground flex items-center gap-2 text-sm font-normal">
          <Switch checked={onlyUnconfirmed} onCheckedChange={setOnlyUnconfirmed} />
          {t('followedDocs.feed.onlyUnconfirmed')}
        </Label>
      </div>

      {filtered.length === 0 ? (
        <p className="text-muted-foreground py-8 text-center text-sm">
          {t('followedDocs.feed.emptyFiltered')}
        </p>
      ) : (
        <ItemGroup className="gap-1.5">
          {filtered.map((change, i) => {
            const unread = !change.confirmed_on
            const related = change.change_type === 'related_change'
            const Icon = related ? Link2 : FileText
            return (
              <FeedItem
                key={`${change.documentUuid}-${change.detected_on}-${i}`}
                unread={unread}
                icon={
                  <Icon className={unread ? 'text-primary' : 'text-muted-foreground'} />
                }
                title={change.documentName}
                description={plainSnippet(change.description_md)}
                date={formatRelative(change.detected_on)}
                dateTitle={formatDateTime(change.detected_on)}
                onClick={() => onSelectDocument(change.documentUuid)}
              />
            )
          })}
        </ItemGroup>
      )}
    </div>
  )
}
