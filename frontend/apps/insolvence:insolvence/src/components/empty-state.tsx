import { useTranslation } from 'react-i18next'
import { MessageSquareText, Plus } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { PickSubjectArt, WatchRegisterArt } from './illustrations'

/** Opens a new CODEXIS conversation pre-filled with an example request. */
function ConversationHint() {
  const { t } = useTranslation()
  return (
    <div className="border-border/70 mt-8 w-full max-w-sm border-t pt-6">
      <p className="text-muted-foreground text-sm">{t('empty.chatLead')}</p>
      <a
        href={`/?draft=${encodeURIComponent(t('empty.chatPrompt'))}`}
        target="_top"
        className="group border-brand-100 bg-secondary/60 hover:bg-secondary mt-2 flex items-start gap-2.5 rounded-xl border px-3.5 py-3 text-left text-sm transition-colors"
        data-testid="empty-chat-example"
      >
        <MessageSquareText className="text-primary mt-0.5 size-4 shrink-0" />
        <span className="text-foreground/90 italic">“{t('empty.chatPrompt')}”</span>
      </a>
    </div>
  )
}

export function EmptySubjects({ onAdd }: { onAdd: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="flex max-w-md flex-col items-center text-center">
        <WatchRegisterArt className="mb-6 h-44 w-auto" />
        <p className="text-primary mb-3 text-xs font-semibold tracking-[0.14em] uppercase">
          {t('app.title')}
        </p>
        <h2 className="font-display text-foreground text-3xl leading-tight text-balance">
          {t('empty.title')}
        </h2>
        <p className="text-muted-foreground mt-3 text-[15px] leading-relaxed text-pretty">
          {t('empty.text')}
        </p>
        <Button onClick={onAdd} size="lg" className="mt-7" data-testid="empty-add">
          <Plus className="size-4" />
          {t('empty.add')}
        </Button>
        <ConversationHint />
      </div>
    </div>
  )
}

export function NoSelection() {
  const { t } = useTranslation()
  return (
    <div className="text-muted-foreground hidden h-full flex-col items-center justify-center gap-4 p-8 text-center md:flex">
      <PickSubjectArt className="h-44 w-auto" />
      <p className="max-w-[16rem] text-sm text-pretty">{t('empty.noSelection')}</p>
    </div>
  )
}
