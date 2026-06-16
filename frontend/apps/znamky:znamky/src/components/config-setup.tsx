import { useTranslation } from 'react-i18next'
import { KeyRound } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'

/** Landing shown until the user has configured their EUIPO API credentials. */
export function ConfigSetup({ onConfigure }: { onConfigure: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="flex max-w-md flex-col items-center text-center">
        <span className="bg-secondary text-primary ring-brand-100 flex size-14 items-center justify-center rounded-2xl ring-1">
          <KeyRound className="size-7" strokeWidth={1.75} />
        </span>
        <p className="text-primary mt-5 mb-2 text-xs font-semibold tracking-[0.14em] uppercase">
          {t('app.title')}
        </p>
        <h2 className="font-display text-foreground text-3xl leading-tight text-balance">
          {t('config.title')}
        </h2>
        <p className="text-muted-foreground mt-3 text-[15px] leading-relaxed text-pretty">
          {t('config.text')}
        </p>
        <Button onClick={onConfigure} size="lg" className="mt-7" data-testid="config-setup-btn">
          <KeyRound className="size-4" />
          {t('config.action')}
        </Button>
        <p className="text-muted-foreground mt-4 max-w-sm text-xs text-pretty">{t('config.hint')}</p>
      </div>
    </div>
  )
}
