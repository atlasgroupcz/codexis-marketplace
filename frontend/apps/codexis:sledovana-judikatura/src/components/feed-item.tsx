import {
  Item,
  ItemContent,
  ItemDescription,
  ItemMedia,
  ItemTitle,
} from '@workspace/ui/components/item'
import type { ReactNode } from 'react'

interface FeedItemProps {
  /** Type icon (already coloured by the caller); conveys the kind of entry. */
  icon: ReactNode
  /** Unread / unconfirmed → filled dot before the time + subtle emphasis. */
  unread?: boolean
  title: string
  description?: string
  date: string
  /** Absolute timestamp shown on hover of the relative date. */
  dateTitle?: string
  onClick: () => void
}

/** One readable row on the monitoring feed - shadcn Item, type shown by icon. */
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
      {/* Leading column: unread dot then relative time. The dot's space is always
          reserved (transparent when read) so the time never shifts horizontally. */}
      <div className="flex w-28 shrink-0 items-start gap-1.5 self-start pt-1.5">
        <span
          aria-hidden
          className={`mt-1 size-2 shrink-0 rounded-full ${unread ? 'bg-primary' : 'bg-transparent'}`}
        />
        <time
          className="text-muted-foreground text-xs leading-tight tabular-nums"
          title={dateTitle}
        >
          {date}
        </time>
      </div>
      <ItemMedia variant="icon" className={unread ? 'border-primary/30 bg-secondary' : ''}>
        {icon}
      </ItemMedia>
      <ItemContent>
        <ItemTitle>{title}</ItemTitle>
        {description && <ItemDescription>{description}</ItemDescription>}
      </ItemContent>
    </Item>
  )
}
