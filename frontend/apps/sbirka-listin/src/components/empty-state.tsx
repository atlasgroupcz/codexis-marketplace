import { ConversationHint } from '@workspace/ui/components/conversation-hint'
import { useTranslation } from 'react-i18next'
import { AddSubjektForm } from './add-subjekt-form'

/** Collection-of-deeds motif - stacked filings with one lifted, highlighted sheet. */
function DeedsArt({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 200 150" fill="none" className={className} aria-hidden role="img">
      {/* back sheets */}
      <rect
        x="52"
        y="30"
        width="96"
        height="104"
        rx="6"
        fill="#f1f1ed"
        stroke="#e2e2db"
        strokeWidth="1.5"
      />
      <rect
        x="44"
        y="24"
        width="96"
        height="104"
        rx="6"
        fill="#f6f6f4"
        stroke="#e2e2db"
        strokeWidth="1.5"
      />
      {/* highlighted front sheet */}
      <rect
        x="36"
        y="18"
        width="96"
        height="104"
        rx="6"
        fill="#f4f7f0"
        stroke="#cfe3ad"
        strokeWidth="1.5"
      />
      {/* text lines on the front sheet */}
      <rect x="48" y="34" width="60" height="6" rx="3" fill="#cfe3ad" />
      <rect x="48" y="50" width="72" height="4" rx="2" fill="#e2e2db" />
      <rect x="48" y="62" width="72" height="4" rx="2" fill="#e2e2db" />
      <rect x="48" y="74" width="52" height="4" rx="2" fill="#e2e2db" />
      <rect x="48" y="94" width="40" height="4" rx="2" fill="#e2e2db" />
      {/* filing badge */}
      <circle cx="120" cy="100" r="14" fill="#5ea500" stroke="#ffffff" strokeWidth="2" />
      <path
        d="M114 100 l4 4 l8 -8"
        stroke="#ffffff"
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

/** Full-screen state shown when the user tracks no subjects yet. */
export function EmptySubjekty({ onAdded }: { onAdded: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="flex h-full items-center justify-center overflow-y-auto p-6">
      <div className="flex max-w-md flex-col items-center text-center">
        <DeedsArt className="mb-6 h-40 w-auto" />
        <h2 className="font-display text-foreground text-2xl leading-tight text-balance">
          {t('subjekty.emptyTitle')}
        </h2>
        <p className="text-muted-foreground mt-3 text-[15px] leading-relaxed text-pretty">
          {t('subjekty.emptyText')}
        </p>
        <div className="mt-7">
          <AddSubjektForm onAdded={onAdded} />
        </div>
        <ConversationHint
          lead={t('subjekty.chatLead')}
          prompt={t('subjekty.chatPrompt')}
          className="mt-8 w-full max-w-sm"
        />
      </div>
    </div>
  )
}

/** Shown in the detail pane on desktop when no subject is selected. */
export function NoSelection() {
  const { t } = useTranslation()
  return (
    <div className="text-muted-foreground hidden h-full flex-col items-center justify-center gap-4 p-8 text-center md:flex">
      <DeedsArt className="h-40 w-auto opacity-90" />
      <p className="max-w-[16rem] text-sm text-pretty">{t('subjekty.selectHint')}</p>
    </div>
  )
}
