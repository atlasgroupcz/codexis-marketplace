import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemMedia,
  ItemTitle,
} from '@workspace/ui/components/item'
import type { ReactNode } from 'react'

interface FeedItemProps {
  /** Type icon (already coloured by the caller); conveys the kind of entry. */
  icon: ReactNode
  /** Unread / unconfirmed → subtle emphasis + dot, no text badge. */
  unread?: boolean
  title: string
  description?: string
  date: string
  /** Absolute timestamp shown on hover of the relative date. */
  dateTitle?: string
  onClick: () => void
}

/** One readable row on the monitoring feed — shadcn Item, type shown by icon. */
export function FeedItem({
  icon,
  unread,
  title,
  description,
  date,
  dateTitle,
  onClick,
}: FeedItemProps) {
  return (
    <Item
      size="sm"
      variant={unread ? 'muted' : 'default'}
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          onClick()
        }
      }}
      className="hover:bg-accent/60 cursor-pointer"
    >
      <time
        className="text-muted-foreground w-28 shrink-0 self-start pt-1.5 text-xs leading-tight tabular-nums"
        title={dateTitle}
      >
        {date}
      </time>
      <ItemMedia variant="icon" className={unread ? 'border-primary/30 bg-secondary' : ''}>
        {icon}
      </ItemMedia>
      <ItemContent>
        <ItemTitle>{title}</ItemTitle>
        {description && <ItemDescription>{description}</ItemDescription>}
      </ItemContent>
      {unread && (
        <ItemActions className="self-start pt-2">
          <span aria-hidden className="bg-primary size-2 rounded-full" />
        </ItemActions>
      )}
    </Item>
  )
}
