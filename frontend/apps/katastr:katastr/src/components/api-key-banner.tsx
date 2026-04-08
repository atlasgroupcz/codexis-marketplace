import { useTranslation } from 'react-i18next'
import { AlertTriangle } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'

interface ApiKeyBannerProps {
  onOpenSettings: () => void
}

export function ApiKeyBanner({ onOpenSettings }: ApiKeyBannerProps) {
  const { t } = useTranslation()
  return (
    <div className="flex flex-col gap-3 rounded-lg border border-amber-300 bg-amber-50 p-4 text-amber-900 dark:border-amber-800 dark:bg-amber-950/50 dark:text-amber-200 sm:flex-row sm:items-center sm:justify-between">
      <div className="flex gap-3">
        <AlertTriangle className="mt-0.5 size-5 shrink-0" />
        <div className="text-sm">
          <p className="font-medium">{t('apiKeyBanner.title')}</p>
          <p className="mt-1">{t('apiKeyBanner.text')}</p>
        </div>
      </div>
      <Button onClick={onOpenSettings} className="shrink-0" data-testid="banner-open-settings">
        {t('apiKeyBanner.action')}
      </Button>
    </div>
  )
}
