import { useTranslation } from 'react-i18next'
import { ArrowRight, CalendarCheck, Mail, MessageSquareText, Scale } from 'lucide-react'
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@workspace/ui/components/empty'
import { NoReportsArt, WatchCaseLawArt } from './illustrations'

/** Opens a new CODEXIS conversation with an editable example request prefilled. */
function ConversationHint() {
  const { t } = useTranslation()
  return (
    <div className="border-border/70 mt-10 w-full max-w-sm border-t pt-6">
      <p className="text-muted-foreground text-sm">{t('judikatura.landing.chatLead')}</p>
      <a
        href={`/?draft=${encodeURIComponent(t('judikatura.landing.chatPrompt'))}`}
        target="_top"
        className="group border-brand-100 bg-secondary/60 hover:bg-secondary hover:border-brand-200 mt-2 flex items-start gap-2.5 rounded-xl border px-3.5 py-3 text-left text-sm transition-colors"
        data-testid="empty-chat-example"
      >
        <MessageSquareText className="text-primary mt-0.5 size-4 shrink-0" />
        <span className="text-foreground/90 flex-1 italic">“{t('judikatura.landing.chatPrompt')}”</span>
        <ArrowRight className="text-primary size-4 shrink-0 self-center transition-transform group-hover:translate-x-0.5" />
      </a>
    </div>
  )
}

/** Landing screen when no topics are tracked yet - explains what the app does. */
export function EmptyTopics() {
  const { t } = useTranslation()
  const features = [
    { Icon: Scale, label: t('judikatura.landing.feature1') },
    { Icon: CalendarCheck, label: t('judikatura.landing.feature2') },
    { Icon: Mail, label: t('judikatura.landing.feature3') },
  ]
  const steps = [
    { title: t('judikatura.landing.step1Title'), text: t('judikatura.landing.step1Text') },
    { title: t('judikatura.landing.step2Title'), text: t('judikatura.landing.step2Text') },
    { title: t('judikatura.landing.step3Title'), text: t('judikatura.landing.step3Text') },
  ]
  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto flex min-h-full max-w-xl flex-col items-center justify-center px-6 py-12 text-center">
        <WatchCaseLawArt className="h-40 w-auto" />
        <h2 className="font-display text-foreground mt-6 text-3xl leading-tight text-balance sm:text-4xl">
          {t('judikatura.landing.title')}
        </h2>
        <p className="text-muted-foreground mt-3 max-w-md text-[15px] leading-relaxed text-pretty">
          {t('judikatura.landing.text')}
        </p>

        <div className="mt-6 flex flex-wrap items-center justify-center gap-2">
          {features.map(({ Icon, label }) => (
            <span
              key={label}
              className="bg-secondary text-secondary-foreground inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-sm"
            >
              <Icon className="text-primary size-3.5" />
              {label}
            </span>
          ))}
        </div>

        <ol className="mt-10 w-full max-w-md space-y-5 text-left">
          {steps.map((step, index) => (
            <li key={step.title} className="flex items-start gap-4">
              <span className="bg-secondary text-primary ring-brand-100 flex size-8 shrink-0 items-center justify-center rounded-full text-sm font-semibold ring-1">
                {index + 1}
              </span>
              <div className="pt-0.5">
                <p className="text-foreground font-medium">{step.title}</p>
                <p className="text-muted-foreground mt-0.5 text-sm leading-relaxed">{step.text}</p>
              </div>
            </li>
          ))}
        </ol>

        <ConversationHint />
      </div>
    </div>
  )
}

/** Feed with no reports yet (topics ARE tracked). */
export function NoReports() {
  const { t } = useTranslation()
  return (
    <Empty className="border-none py-12">
      <EmptyHeader>
        <EmptyMedia>
          <NoReportsArt className="h-32 w-auto" />
        </EmptyMedia>
        <EmptyTitle>{t('judikatura.feed.empty')}</EmptyTitle>
        <EmptyDescription>{t('judikatura.feed.emptyHint')}</EmptyDescription>
      </EmptyHeader>
    </Empty>
  )
}
