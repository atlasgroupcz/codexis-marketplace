import { useTranslation } from 'react-i18next'
import { Button } from '@workspace/ui/components/button'

interface ErrorMessageProps {
  error: Error
  onRetry?: () => void
}

export function ErrorMessage({ error, onRetry }: ErrorMessageProps) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-col items-center gap-4 py-12 text-center">
      <p className="text-sm text-destructive">{error.message}</p>
      {onRetry && (
        <Button variant="outline" size="sm" onClick={onRetry}>
          {t('judikatura.retry')}
        </Button>
      )}
    </div>
  )
}
