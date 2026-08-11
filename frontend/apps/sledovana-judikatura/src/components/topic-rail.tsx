import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemTitle,
} from '@workspace/ui/components/item'
import { cn } from '@workspace/ui/lib/utils'
import { useTranslation } from 'react-i18next'
import type { TopicSummary } from '@/lib/schemas'

interface TopicRailProps {
  topics: Array<TopicSummary>
  topicFilter: string | null
  onSelectFilter: (uuid: string | null) => void
}

function UnreadPill({ count }: { count: number }) {
  return (
    <span className="bg-primary text-primary-foreground flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold tabular-nums">
      {count}
    </span>
  )
}

export function TopicRail({ topics, topicFilter, onSelectFilter }: TopicRailProps) {
  const { t } = useTranslation()
  const totalUnconfirmed = topics.reduce((sum, topic) => sum + topic.unconfirmed_reports, 0)

  const railItem = (
    key: string,
    active: boolean,
    onSelect: () => void,
    title: string,
    description: string | null,
    unread: number,
  ) => (
    <Item
      key={key}
      size="sm"
      role="button"
      tabIndex={0}
      onClick={onSelect}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          onSelect()
        }
      }}
      className={cn(
        'cursor-pointer',
        active ? 'border-primary/40 bg-accent' : 'hover:bg-accent/50',
      )}
    >
      <ItemContent>
        <ItemTitle className="line-clamp-1">{title}</ItemTitle>
        {description && <ItemDescription className="line-clamp-1">{description}</ItemDescription>}
      </ItemContent>
      {unread > 0 && (
        <ItemActions>
          <UnreadPill count={unread} />
        </ItemActions>
      )}
    </Item>
  )

  return (
    <ItemGroup className="gap-1">
      {railItem(
        '__all__',
        !topicFilter,
        () => onSelectFilter(null),
        t('judikatura.allTopics'),
        null,
        totalUnconfirmed,
      )}
      {topics.map((topic) =>
        railItem(
          topic.uuid,
          topicFilter === topic.uuid,
          () => onSelectFilter(topic.uuid),
          topic.name,
          t('judikatura.areasCount', { count: topic.areas }),
          topic.unconfirmed_reports,
        ),
      )}
    </ItemGroup>
  )
}
