import { useTranslation } from 'react-i18next'
import { ExternalLink, KeyRound } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'

/** Full-screen landing shown until a ČÚZK API key is configured. */
export function ConfigSetup({ onConfigure }: { onConfigure: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="flex max-w-md flex-col items-center text-center">
        <span className="bg-secondary text-primary ring-brand-100 flex size-14 items-center justify-center rounded-2xl ring-1">
          <KeyRound className="size-7" strokeWidth={1.75} />
        </span>
        <h2 className="font-display text-foreground mt-5 text-2xl leading-tight text-balance">
          {t('apiKeyBanner.title')}
        </h2>
        <p className="text-muted-foreground mt-3 text-[15px] leading-relaxed text-pretty">
          {t('apiKeyBanner.text')}
        </p>
        <Button onClick={onConfigure} size="lg" className="mt-7" data-testid="config-setup-btn">
          <KeyRound className="size-4" />
          {t('apiKeyBanner.action')}
        </Button>
        <a
          href="https://registrace.cuzk.gov.cz/"
          target="_blank"
          rel="noreferrer"
          className="text-primary mt-4 inline-flex items-center gap-1 text-sm hover:underline"
          data-testid="config-get-key-link"
        >
          <ExternalLink className="size-3.5" />
          {t('settings.getKey')}
        </a>
        <p className="text-muted-foreground mt-2 max-w-xs text-xs text-pretty">
          {t('settings.getKeyHint')}
        </p>
      </div>
    </div>
  )
}
