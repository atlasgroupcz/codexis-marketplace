import { useTranslation } from 'react-i18next'
import { MessageSquareText } from 'lucide-react'
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@workspace/ui/components/empty'
import { NoChangesArt, WatchDocsArt } from './illustrations'

/** Opens a new CODEXIS conversation pre-filled with an example request. */
function ConversationHint() {
  const { t } = useTranslation()
  return (
    <div className="border-border/70 w-full max-w-sm border-t pt-6">
      <p className="text-muted-foreground text-sm">{t('followedDocs.landing.chatLead')}</p>
      <a
        href={`/?prompt=${encodeURIComponent(t('followedDocs.landing.chatPrompt'))}`}
        target="_top"
        className="group border-brand-100 bg-secondary/60 hover:bg-secondary mt-2 flex items-start gap-2.5 rounded-xl border px-3.5 py-3 text-left text-sm transition-colors"
        data-testid="empty-chat-example"
      >
        <MessageSquareText className="text-primary mt-0.5 size-4 shrink-0" />
        <span className="text-foreground/90 italic">“{t('followedDocs.landing.chatPrompt')}”</span>
      </a>
    </div>
  )
}

/** Landing screen when nothing is tracked yet. */
export function EmptyDocuments() {
  const { t } = useTranslation()
  return (
    <Empty className="h-full border-none">
      <EmptyHeader>
        <EmptyMedia>
          <WatchDocsArt className="h-44 w-auto" />
        </EmptyMedia>
        <p className="text-primary text-xs font-semibold tracking-[0.14em] uppercase">
          {t('followedDocs.title')}
        </p>
        <EmptyTitle className="font-display text-3xl leading-tight">
          {t('followedDocs.landing.title')}
        </EmptyTitle>
        <EmptyDescription className="text-[15px]">
          {t('followedDocs.landing.text')}
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <ConversationHint />
      </EmptyContent>
    </Empty>
  )
}

/** Feed with no changes detected yet (something IS tracked). */
export function NoChanges() {
  const { t } = useTranslation()
  return (
    <Empty className="border-none py-12">
      <EmptyHeader>
        <EmptyMedia>
          <NoChangesArt className="h-32 w-auto" />
        </EmptyMedia>
        <EmptyTitle>{t('followedDocs.feed.empty')}</EmptyTitle>
        <EmptyDescription>{t('followedDocs.feed.emptyHint')}</EmptyDescription>
      </EmptyHeader>
    </Empty>
  )
}
