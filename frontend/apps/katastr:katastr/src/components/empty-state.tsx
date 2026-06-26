import { useTranslation } from 'react-i18next'
import { MessageSquareText } from 'lucide-react'
import { AddProceedingForm } from './add-proceeding-form'

/** Cadastral parcel-map motif — parcels, a boundary line and a located plot. */
function PickProceedingArt({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 200 150" fill="none" className={className} aria-hidden role="img">
      {/* parcels */}
      <path d="M20 30 L92 22 L100 70 L28 80 Z" fill="#f6f6f4" stroke="#e2e2db" strokeWidth="1.5" />
      <path d="M92 22 L168 32 L160 78 L100 70 Z" fill="#f1f1ed" stroke="#e2e2db" strokeWidth="1.5" />
      <path d="M100 70 L160 78 L168 120 L108 126 Z" fill="#f6f6f4" stroke="#e2e2db" strokeWidth="1.5" />
      {/* highlighted / located parcel */}
      <path d="M28 80 L100 70 L108 126 L36 134 Z" fill="#f4f7f0" stroke="#cfe3ad" strokeWidth="1.5" />
      {/* surveyor boundary line */}
      <path d="M8 62 L96 50 L190 62" stroke="#dcdcd5" strokeWidth="1.5" strokeDasharray="4 5" />
      {/* parcel-number faux labels */}
      <rect x="46" y="44" width="20" height="5" rx="2.5" fill="#e2e2db" />
      <rect x="118" y="46" width="20" height="5" rx="2.5" fill="#e2e2db" />
      <rect x="124" y="96" width="20" height="5" rx="2.5" fill="#e2e2db" />
      <rect x="78" y="110" width="22" height="5" rx="2.5" fill="#bcd79a" />
      {/* location pin on the highlighted parcel */}
      <path
        d="M66 108 C60 100 58 97 58 92 a8 8 0 1 1 16 0 c0 5 -2 8 -8 16 Z"
        fill="#5ea500"
        stroke="#ffffff"
        strokeWidth="2"
        strokeLinejoin="round"
      />
      <circle cx="66" cy="92" r="2.75" fill="#ffffff" />
    </svg>
  )
}

/** Opens a new CODEXIS conversation pre-filled with an example request. */
function ConversationHint() {
  const { t } = useTranslation()
  return (
    <div className="border-border/70 mt-8 w-full max-w-sm border-t pt-6">
      <p className="text-muted-foreground text-sm">{t('proceedings.chatLead')}</p>
      <a
        href={`/?prompt=${encodeURIComponent(t('proceedings.chatPrompt'))}`}
        target="_top"
        className="group border-brand-100 bg-secondary/60 hover:bg-secondary mt-2 flex items-start gap-2.5 rounded-xl border px-3.5 py-3 text-left text-sm transition-colors"
        data-testid="empty-chat-example"
      >
        <MessageSquareText className="text-primary mt-0.5 size-4 shrink-0" />
        <span className="text-foreground/90 italic">“{t('proceedings.chatPrompt')}”</span>
      </a>
    </div>
  )
}

/** Full-screen state shown when the user tracks no proceedings yet. */
export function EmptyProceedings({ onAdded }: { onAdded: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="flex h-full items-center justify-center overflow-y-auto p-6">
      <div className="flex max-w-md flex-col items-center text-center">
        <PickProceedingArt className="mb-6 h-40 w-auto" />
        <h2 className="font-display text-foreground text-2xl leading-tight text-balance">
          {t('proceedings.emptyTitle')}
        </h2>
        <p className="text-muted-foreground mt-3 text-[15px] leading-relaxed text-pretty">
          {t('proceedings.emptyText')}
        </p>
        <div className="mt-7">
          <AddProceedingForm onAdded={onAdded} />
        </div>
        <ConversationHint />
      </div>
    </div>
  )
}

/** Shown in the detail pane on desktop when no proceeding is selected. */
export function NoSelection() {
  const { t } = useTranslation()
  return (
    <div className="text-muted-foreground hidden h-full flex-col items-center justify-center gap-4 p-8 text-center md:flex">
      <PickProceedingArt className="h-40 w-auto opacity-90" />
      <p className="max-w-[16rem] text-sm text-pretty">{t('proceedings.selectHint')}</p>
    </div>
  )
}
